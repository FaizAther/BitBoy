#!/usr/bin/env python3
"""SoftGBC Tetris agent: Darwin menu + gameplay races → champion play + GDB.

Single entry point (do not add sibling race/scout scripts):

  ./play_agent.sh                      # forever on softgbcBot1
  ./play_agent.sh --darwin             # evolve genomes, promote winner, then play
  python3 play_agent.py --darwin --bots 4 --generations 3 --gen-seconds 25 --race-seconds 45
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import socket
import sys
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

DOCS = Path(__file__).resolve().parent
DEFAULT_LOG = DOCS / "play_agent.log"
GENOME_PATH = DOCS / "play_agent_genome.json"
TETRIS = "Tetris DX (World) (SGB Enhanced) (GB Compatible).gbc"

RIGHT, LEFT, UP, DOWN = 1 << 0, 1 << 1, 1 << 2, 1 << 3
A, B, SELECT, START = 1 << 4, 1 << 5, 1 << 6, 1 << 7

PF_X0, PF_Y0 = 16, 16
PF_COLS, PF_ROWS = 10, 18
CELL = 8
SCREEN_W, SCREEN_H = 160, 144


class PlayClient:
    def __init__(self, base: str, session: str):
        self.base = base.rstrip("/")
        self.session = session

    def _req(
        self,
        method: str,
        path: str,
        body: bytes | None = None,
        content_type: str | None = None,
        timeout: float = 10.0,
        raw: bool = False,
    ) -> tuple[int, object]:
        url = f"{self.base}{path}"
        sep = "&" if "?" in path else "?"
        if "s=" not in path:
            url = f"{url}{sep}s={self.session}"
        headers = {"X-Play-Session": self.session}
        if not raw:
            headers["Accept"] = "application/json"
        if content_type:
            headers["Content-Type"] = content_type
        req = urllib.request.Request(url, data=body, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                data = resp.read()
                code = resp.getcode()
        except urllib.error.HTTPError as e:
            data = e.read()
            code = e.code
        except urllib.error.URLError as e:
            return 0, {"error": str(e.reason)}
        except (TimeoutError, TimeoutError, OSError) as e:
            # socket.timeout / http read stalls during GDB halt contention
            return 0, {"error": f"timeout:{e}"}
        if raw:
            return code, data
        if not data:
            return code, {}
        try:
            return code, json.loads(data.decode("utf-8"))
        except json.JSONDecodeError:
            return code, {"raw": data[:200].decode("utf-8", errors="replace")}

    def load_builtin(self, name: str) -> tuple[int, object]:
        return self._req(
            "POST",
            "/api/play/load/builtin",
            body=json.dumps({"name": name}).encode(),
            content_type="application/json",
            timeout=60.0,
        )

    def input_bits(self, bits: int) -> tuple[int, object]:
        # softgbc-play parses {"bits":N} with NO space after ':' — default
        # json.dumps spaces break input (400 bad json) so the agent never moves.
        body = json.dumps({"bits": bits & 0xFF}, separators=(",", ":")).encode()
        return self._req(
            "POST",
            "/api/play/input",
            body=body,
            content_type="application/json",
            timeout=5.0,
        )

    def status(self) -> tuple[int, object]:
        return self._req("GET", "/api/play/status", timeout=5.0)

    def debug(self) -> tuple[int, object]:
        return self._req("GET", "/api/play/debug", timeout=5.0)

    def sessions(self) -> tuple[int, object]:
        return self._req("GET", "/api/play/sessions", timeout=5.0)

    def stop(self) -> tuple[int, object]:
        return self._req("POST", "/api/play/stop", body=b"", timeout=10.0)

    def frame_rgb(self) -> tuple[int, bytes]:
        code, data = self._req("GET", "/api/play/frame.rgb", timeout=3.0, raw=True)
        return code, data if isinstance(data, (bytes, bytearray)) else b""

    def gdb_port_for_self(self) -> int:
        """Prefer /sessions (no emu lock) over /status."""
        code, data = self.sessions()
        if code == 200 and isinstance(data, dict):
            for s in data.get("sessions") or []:
                if s.get("id") == self.session:
                    return int(s.get("gdb_port") or 0)
        code, st = self.status()
        if code == 200 and isinstance(st, dict):
            return int(st.get("gdb_port") or 0)
        return 0


# --- GDB RSP (brief attach; samples SM83 then detaches so play can resume) ---


def _checksum(payload: bytes) -> int:
    return sum(payload) & 0xFF


def _decode_rle(body: bytes) -> bytes:
    """Decode GDB run-length encoding in packet bodies."""
    out = bytearray()
    i = 0
    while i < len(body):
        b = body[i]
        if b == ord("*") and out and i + 1 < len(body):
            # repeat previous byte (n - 29) times; n is next char
            n = body[i + 1] - 29
            if n > 0:
                out.extend([out[-1]] * n)
            i += 2
            continue
        out.append(b)
        i += 1
    return bytes(out)


class GdbClient:
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port

    def sample(self) -> dict:
        """Attach, read regs + 4 bytes at PC, detach. Raises on failure."""
        s = socket.create_connection((self.host, self.port), timeout=2.0)
        s.settimeout(2.0)
        try:
            s.sendall(b"+")
            self._send(s, "qSupported:swbreak+")
            self._recv(s)
            s.sendall(b"+")
            self._send(s, "Hg0")
            self._recv(s)
            s.sendall(b"+")
            self._send(s, "g")
            g_raw = self._recv(s)
            s.sendall(b"+")
            regs = self._parse_g(g_raw)
            pc = regs["pc"]
            self._send(s, f"m{pc:x},4")
            m_raw = self._recv(s)
            s.sendall(b"+")
            mem = self._parse_m(m_raw)
            # Detach cleanly so softgbc-play clears gdb_halted.
            self._send(s, "D")
            try:
                self._recv(s)
            except Exception:
                pass
            return {"regs": regs, "mem_at_pc": mem, "raw_g": g_raw[:80]}
        finally:
            try:
                s.close()
            except Exception:
                pass

    def _send(self, sock: socket.socket, payload: str) -> None:
        data = payload.encode()
        sock.sendall(b"$" + data + b"#" + f"{_checksum(data):02x}".encode())

    def _recv(self, sock: socket.socket) -> bytes:
        # Skip acks until '$'
        while True:
            b = sock.recv(1)
            if not b:
                raise EOFError("gdb eof")
            if b == b"$":
                break
        buf = bytearray(b"$")
        while True:
            b = sock.recv(1)
            if not b:
                raise EOFError("gdb eof mid-pkt")
            buf.extend(b)
            if len(buf) >= 4 and buf[-3] == ord("#"):
                break
        # body between $ and #
        body = bytes(buf[1:-3])
        return _decode_rle(body)

    def _parse_g(self, body: bytes) -> dict:
        # Expect hex digits for 16×u32 LE (i386 layout).
        hexbody = body
        if hexbody[:1] in (b"E", b"e") and len(hexbody) < 8:
            raise RuntimeError(f"gdb g error: {hexbody!r}")
        # Strip non-hex
        hx = "".join(chr(c) for c in hexbody if chr(c) in "0123456789abcdefABCDEF")
        raw = bytes.fromhex(hx)
        if len(raw) < 40:
            raise RuntimeError(f"short g packet ({len(raw)} bytes)")

        def u32(off: int) -> int:
            return int.from_bytes(raw[off : off + 4], "little")

        return {
            "af": u32(0) & 0xFFFF,
            "bc": u32(4) & 0xFFFF,
            "de": u32(8) & 0xFFFF,
            "hl": u32(12) & 0xFFFF,
            "sp": u32(16) & 0xFFFF,
            "pc": u32(32) & 0xFFFF,
            "ime": bool(u32(36)),
        }

    def _parse_m(self, body: bytes) -> list[int]:
        hx = "".join(chr(c) for c in body if chr(c) in "0123456789abcdefABCDEF")
        raw = bytes.fromhex(hx) if hx else b""
        return list(raw)


def log_line(fp, msg: str) -> None:
    line = f"{time.strftime('%H:%M:%S')} {msg}"
    print(line, flush=True)
    fp.write(line + "\n")
    fp.flush()


def hold(client: PlayClient, bits: int, hold_s: float) -> None:
    client.input_bits(bits)
    time.sleep(hold_s)
    client.input_bits(0)


def tap(client: PlayClient, bits: int, hold_s: float = 0.06, gap_s: float = 0.08) -> None:
    hold(client, bits, hold_s)
    time.sleep(gap_s)


def _has_marathon_well(rgb: bytes) -> bool:
    """Brick playfield wall + light HUD — shared by LIVE and DEMO."""
    # Left brick wall ~x=8..14: many dark pixels.
    dark = 0
    for y in range(24, 120, 2):
        i = (y * SCREEN_W + 10) * 3
        if rgb[i] < 100:
            dark += 1
    # Right HUD panel is lighter (score) — sample x=130.
    right_light = 0
    for y in range(24, 80, 2):
        i = (y * SCREEN_W + 130) * 3
        if rgb[i] > 180:
            right_light += 1
    return dark >= 24 and right_light >= 8


def in_gameplay_screen(rgb: bytes) -> bool:
    """True when the Marathon playfield brick well is visible (not title/menus)."""
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return False
    # Check well FIRST — title heuristic false-positives on the brick+white
    # top-left of an empty Marathon start (uniq≤6 mid-gray), which made the
    # agent think LIVE was the title and mash Start / abandon the game.
    return _has_marathon_well(rgb)


def looks_like_demo(rgb: bytes) -> bool:
    """Attract DEMO: wide bottom garbage + mid-well badge ink.

    Empty Marathon start must not match. A LIVE well with a tall *narrow*
    stack (normal play) also must not match — that false DEMO made the agent
    hit Start and dump a real game.
    """
    if not in_gameplay_screen(rgb):
        return False
    # Attract garbage is wide across the well; a single-column LIVE stack is not.
    cols_hit = 0
    for col in range(PF_COLS):
        x = PF_X0 + col * CELL + CELL // 2
        dark = 0
        for y in range(100, 130):
            if rgb[(y * SCREEN_W + x) * 3] < 120:
                dark += 1
        if dark >= 8:
            cols_hit += 1
    if cols_hit < 6:
        return False
    dark = 0
    total = 0
    for y in range(36, 58):
        for x in range(48, 104):
            i = (y * SCREEN_W + x) * 3
            total += 1
            if rgb[i] < 140:
                dark += 1
    ratio = dark / max(total, 1)
    return 0.10 <= ratio <= 0.45


def exit_demo_if_needed(client: PlayClient, fp, rgb: bytes | None) -> bool:
    if rgb is None or not looks_like_demo(rgb):
        return False
    log_line(fp, "demo: attract mode detected — Start to exit")
    for _ in range(3):
        tap(client, START, 0.15, 0.4)
    return True


# Menu action set for Darwin genomes (bit → weight index order).
MENU_ACTIONS = (UP, DOWN, LEFT, RIGHT, A, B, START)
STAGE_NAMES = {
    -1: "DEMO",
    0: "boot",
    1: "title",
    2: "name/file",
    3: "mode",
    4: "level",
    5: "LIVE",
}


def _is_title_menu(rgb: bytes) -> bool:
    """Checkerboard title: mid-gray tiles in the top-left, not copyright white."""
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return False
    if _has_marathon_well(rgb):
        return False
    vals = [rgb[(y * SCREEN_W + x) * 3] for y in range(0, 40) for x in range(0, 40)]
    avg = sum(vals) // max(len(vals), 1)
    uniq = len(set(vals))
    return uniq <= 6 and 40 < avg < 180


def _white_frac(rgb: bytes) -> float:
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return 0.0
    n = sum(1 for i in range(0, len(rgb), 3) if rgb[i] > 230)
    return n / (SCREEN_W * SCREEN_H)


def _looks_like_name_keyboard(rgb: bytes) -> bool:
    """Letter grid only — high white bg + dense ink in lower panel."""
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return False
    if _is_title_menu(rgb) or in_gameplay_screen(rgb):
        return False
    if _white_frac(rgb) < 0.62:
        return False
    dark = 0
    for y in range(100, 130):
        for x in range(16, 144):
            if rgb[(y * SCREEN_W + x) * 3] < 90:
                dark += 1
    return dark >= 450


def _looks_like_mode_select(rgb: bytes) -> bool:
    """Marathon/Ultra/40LINES folder cards on gray (not white file menus)."""
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return False
    if _is_title_menu(rgb) or in_gameplay_screen(rgb):
        return False
    wf = _white_frac(rgb)
    # Mode folder is gray; level select is whiter.
    if not (0.18 < wf < 0.50):
        return False
    frames = 0
    for y in (36, 52, 68, 84):
        row = [rgb[(y * SCREEN_W + x) * 3] for x in range(20, 120)]
        if min(row) < 50 and max(row) > 180:
            frames += 1
    return frames >= 2


def _looks_like_level_select(rgb: bytes) -> bool:
    """Marathon LEVEL 0..9 slider screen (whiter than mode folder)."""
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return False
    if _is_title_menu(rgb) or in_gameplay_screen(rgb):
        return False
    wf = _white_frac(rgb)
    if not (0.50 <= wf < 0.78):
        return False
    # Slider / header ink present; not a letter keyboard (those are denser below).
    band = [rgb[(y * SCREEN_W + x) * 3] for y in range(40, 70) for x in range(20, 140)]
    if not band:
        return False
    dark = sum(1 for v in band if v < 60)
    return min(band) < 40 and 30 <= dark <= 400


def classify_stage(rgb: bytes) -> int:
    """Fitness stage: higher is better. DEMO = -1 penalty."""
    if len(rgb) < SCREEN_W * SCREEN_H * 3:
        return 0
    if looks_like_demo(rgb):
        return -1
    if in_gameplay_screen(rgb):
        return 5
    if _is_title_menu(rgb):
        return 1
    # Keyboard before level — both are whitish; mis-order bounced us off LIVE.
    if _looks_like_name_keyboard(rgb):
        return 2
    if _looks_like_mode_select(rgb):
        return 3
    if _looks_like_level_select(rgb):
        return 4
    if _white_frac(rgb) > 0.55:
        return 2
    return 0


def default_genome() -> list[float]:
    # Mild bias: A and UP help menus; keep exploration on the rest.
    return [1.4, 0.8, 0.7, 0.9, 1.6, 0.6, 0.5]


def default_play_weights() -> list[float]:
    # DOWN, LEFT, RIGHT, rotate-A, rotate-B, panic-down.
    return [1.5, 0.9, 0.9, 0.65, 0.45, 1.2]


def load_genome(path: Path = GENOME_PATH) -> dict:
    if path.is_file():
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            w = data.get("weights")
            if isinstance(w, list) and len(w) == len(MENU_ACTIONS):
                pw = data.get("play_weights")
                if not isinstance(pw, list) or len(pw) != len(default_play_weights()):
                    data["play_weights"] = default_play_weights()
                data.setdefault("best_score", 0)
                return data
        except (OSError, json.JSONDecodeError, TypeError):
            pass
    return {
        "weights": default_genome(),
        "fitness": 0,
        "generation": 0,
        "champion": None,
        "play_weights": default_play_weights(),
        "best_score": 0,
    }


def save_genome(data: dict, path: Path = GENOME_PATH) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def mutate_weights(weights: list[float], rng: random.Random, scale: float = 0.35) -> list[float]:
    out = []
    for w in weights:
        nw = max(0.05, w + rng.gauss(0, scale))
        out.append(nw)
    return out


def sample_action(weights: list[float], rng: random.Random) -> int:
    # Softmax over weights → pick MENU_ACTIONS index.
    m = max(weights)
    exps = [math.exp(w - m) for w in weights]
    total = sum(exps) or 1.0
    r = rng.random() * total
    acc = 0.0
    for i, e in enumerate(exps):
        acc += e
        if r <= acc:
            return MENU_ACTIONS[i]
    return MENU_ACTIONS[-1]


def dismiss_copyright(client: PlayClient, budget_s: float = 8.0) -> None:
    t0 = time.monotonic()
    while time.monotonic() - t0 < budget_s:
        code, rgb = client.frame_rgb()
        if code == 200 and isinstance(rgb, (bytes, bytearray)) and _is_title_menu(bytes(rgb)):
            return
        tap(client, A, 0.16, 0.28)


def _name_keyboard_end(client: PlayClient) -> None:
    tap(client, DOWN, 0.22, 0.32)
    tap(client, DOWN, 0.22, 0.32)
    for _ in range(12):
        tap(client, RIGHT, 0.1, 0.14)
    tap(client, A, 0.3, 0.5)
    tap(client, RIGHT, 0.1, 0.14)
    tap(client, A, 0.3, 0.5)


def run_menu_genome(
    client: PlayClient,
    weights: list[float],
    seconds: float,
    rng: random.Random,
) -> tuple[int, int]:
    """Drive menus with genome + stage scripts. Returns (best_stage, final_stage)."""
    dismiss_copyright(client)
    best = 0
    final = 0
    kbd_tries = 0
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        code, rgb = client.frame_rgb()
        if code != 200 or not isinstance(rgb, (bytes, bytearray)):
            tap(client, A, 0.15, 0.3)
            continue
        rgb_b = bytes(rgb)
        if len(rgb_b) < SCREEN_W * SCREEN_H * 3:
            continue
        st = classify_stage(rgb_b)
        final = st
        if st > best:
            best = st
        if st == 5:
            break
        if st == -1:
            # Attract playfield — Start returns to title; keep going toward 1P path.
            tap(client, START, 0.2, 0.4)
            continue
        # Mode / level: confirm hard — short taps can bounce back to title.
        if st == 3:
            tap(client, A, 0.3, 0.65)
            continue
        if st == 4:
            for _ in range(3):
                hold(client, A, 0.35)
                time.sleep(0.55)
            continue
        if st == 0:
            # Blank/transition frames — wait, don't mash directionals.
            tap(client, A, 0.12, 0.4)
            continue
        if _looks_like_name_keyboard(rgb_b):
            kbd_tries += 1
            _name_keyboard_end(client)
            if kbd_tries >= 2:
                for _ in range(3):
                    tap(client, B, 0.2, 0.35)
                for _ in range(4):
                    tap(client, DOWN, 0.18, 0.28)
                tap(client, A, 0.25, 0.5)  # EXIT → title (cursor often 1PLAYER after)
                kbd_tries = 0
            continue
        kbd_tries = 0
        if _is_title_menu(rgb_b):
            for _ in range(3):
                tap(client, UP, 0.2, 0.32)
            tap(client, A, 0.24, 0.5)
            continue
        # File / other text menus: mostly A, occasional UP/RIGHT from genome.
        if rng.random() < 0.65:
            tap(client, A, 0.22, 0.45)
        else:
            tap(client, sample_action(weights, rng), 0.18, 0.32)
    return best, final


def enter_tetris(client: PlayClient, fp, genome: dict | None = None) -> None:
    """Use best learned genome (or default) until LIVE or timeout."""
    g = genome or load_genome()
    weights = list(g.get("weights") or default_genome())
    log_line(
        fp,
        f"menu: Darwin genome gen={g.get('generation', 0)} "
        f"prior_fit={g.get('fitness', 0)} → Marathon",
    )
    code, ack = client.input_bits(0)
    if code != 200:
        log_line(fp, f"menu: WARN input rejected http={code} body={ack}")
    rng = random.Random(0xC0FFEE ^ int(time.time()) & 0xFFFF)
    best, final = run_menu_genome(client, weights, seconds=90.0, rng=rng)
    log_line(
        fp,
        f"menu: stage best={STAGE_NAMES.get(best, best)} "
        f"final={STAGE_NAMES.get(final, final)}",
    )
    if best >= 5:
        log_line(fp, "menu: done — LIVE playfield")
    else:
        log_line(fp, "menu: WARN not LIVE yet — play loop may still recover")


def darwin_evolve(
    base: str,
    bots: int,
    generations: int,
    gen_seconds: float,
    race_seconds: float,
    fp,
    survivors: int = 2,
) -> dict:
    """Multi-bot tournament: reach LIVE, then compete on board fitness."""
    rng = random.Random(0xDA241)
    state = load_genome()
    population = [
        (
            list(state["weights"]),
            list(state.get("play_weights") or default_play_weights()),
        )
    ]
    while len(population) < bots:
        population.append(
            (
                mutate_weights(population[0][0], rng),
                mutate_weights(population[0][1], rng, scale=0.25),
            )
        )

    champion_session = state.get("champion")
    all_sessions: set[str] = set()
    for gen in range(1, generations + 1):
        log_line(
            fp,
            f"darwin: generation {gen}/{generations} bots={bots} "
            f"menu={gen_seconds:.0f}s race={race_seconds:.0f}s",
        )
        scored: list[dict] = []

        def eval_bot(idx_genomes: tuple[int, tuple[list[float], list[float]]]):
            idx, genomes = idx_genomes
            menu_weights, play_weights = genomes
            sid = f"darwin{idx}"
            all_sessions.add(sid)
            client = PlayClient(base, sid)
            code, body = client.load_builtin(TETRIS)
            if code != 200:
                return {
                    "stage": 0,
                    "final": 0,
                    "score": 0,
                    "estimated_lines": 0,
                    "live_snapshots": 0,
                    "menu_weights": menu_weights,
                    "play_weights": play_weights,
                    "id": sid,
                    "error": f"load_fail:{body}",
                }
            brng = random.Random(0xD00000 + gen * 1000 + idx)
            best, final = run_menu_genome(client, menu_weights, gen_seconds, brng)
            race = {"score": 0, "estimated_lines": 0, "live_snapshots": 0}
            if best >= 5:
                race = run_gameplay_race(
                    client,
                    play_weights,
                    race_seconds,
                    brng,
                )
                final = 5 if race.get("live") else final
            return {
                "stage": best if best >= 0 else -1,
                "final": final,
                "score": int(race.get("score") or 0),
                "estimated_lines": int(race.get("estimated_lines") or 0),
                "live_snapshots": int(race.get("live_snapshots") or 0),
                "menu_weights": menu_weights,
                "play_weights": play_weights,
                "id": sid,
                "error": None,
            }

        with ThreadPoolExecutor(max_workers=bots) as pool:
            futs = [
                pool.submit(eval_bot, (i + 1, population[i]))
                for i in range(bots)
            ]
            for fut in as_completed(futs):
                result = fut.result()
                scored.append(result)
                log_line(
                    fp,
                    f"darwin: {result['id']} "
                    f"best={STAGE_NAMES.get(result['stage'], result['stage'])} "
                    f"now={STAGE_NAMES.get(result['final'], result['final'])} "
                    f"score={result['score']} lines~={result['estimated_lines']} "
                    f"samples={result['live_snapshots']}"
                    + (f" err={result['error']}" if result["error"] else ""),
                )

        if not generation_has_valid_bot(scored):
            log_line(fp, "darwin: generation invalid — all bot loads failed; genome unchanged")
            break
        scored.sort(key=race_rank_key, reverse=True)
        keep = max(1, min(survivors, len(scored)))
        elites = scored[:keep]
        champion_session = str(elites[0]["id"])
        state = {
            "weights": elites[0]["menu_weights"],
            "play_weights": elites[0]["play_weights"],
            "fitness": elites[0]["stage"],
            "best_score": elites[0]["score"],
            "estimated_lines": elites[0]["estimated_lines"],
            "generation": int(state.get("generation") or 0) + 1,
            "champion": champion_session,
            "board": [
                {
                    "id": result["id"],
                    "best": STAGE_NAMES.get(result["stage"], result["stage"]),
                    "final": STAGE_NAMES.get(result["final"], result["final"]),
                    "score": result["score"],
                    "estimated_lines": result["estimated_lines"],
                    "live_snapshots": result["live_snapshots"],
                }
                for result in scored
            ],
        }
        save_genome(state)
        log_line(
            fp,
            f"darwin: survivors={[s['id'] for s in elites]} "
            f"champ={champion_session} "
            f"stage={STAGE_NAMES.get(elites[0]['stage'], elites[0]['stage'])} "
            f"score={elites[0]['score']}",
        )
        # Next generation: retain exact winners, then mutate both menu and play policy.
        population = [
            (list(e["menu_weights"]), list(e["play_weights"]))
            for e in elites
        ]
        while len(population) < bots:
            parent = elites[rng.randrange(keep)]
            population.append(
                (
                    mutate_weights(parent["menu_weights"], rng),
                    mutate_weights(parent["play_weights"], rng, scale=0.25),
                )
            )

    # Keep only the promoted winner alive; race session IDs are reused each generation.
    for sid in sorted(all_sessions):
        if sid != champion_session:
            PlayClient(base, sid).stop()
    log_line(fp, f"darwin: cleanup complete; promoted={champion_session}")
    return state


def cell_filled(rgb: bytes, col: int, row: int) -> bool:
    x = PF_X0 + col * CELL + CELL // 2
    y = PF_Y0 + row * CELL + CELL // 2
    if x >= SCREEN_W or y >= SCREEN_H:
        return False
    i = (y * SCREEN_W + x) * 3
    return rgb[i] < 180


def column_heights(rgb: bytes) -> list[int]:
    heights = []
    for c in range(PF_COLS):
        h = 0
        for r in range(PF_ROWS):
            if cell_filled(rgb, c, r):
                h = PF_ROWS - r
                break
        heights.append(h)
    return heights


def board_metrics(rgb: bytes) -> tuple[list[int], int, int]:
    """Return (column heights, covered holes, height bumpiness)."""
    heights = column_heights(rgb)
    holes = 0
    for c in range(PF_COLS):
        seen = False
        for r in range(PF_ROWS):
            filled = cell_filled(rgb, c, r)
            if filled:
                seen = True
            elif seen:
                holes += 1
    bumpiness = sum(abs(heights[i] - heights[i + 1]) for i in range(len(heights) - 1))
    return heights, holes, bumpiness


class BoardFitness:
    """Board-quality race score; rewards survival/clears, penalizes danger."""

    def __init__(self) -> None:
        self.live_snapshots = 0
        self.estimated_lines = 0
        self.score = 0
        self._survival_points = 0
        self._clear_points = 0
        self._previous_aggregate: int | None = None

    def observe(self, heights: list[int], holes: int, bumpiness: int) -> int:
        aggregate = sum(heights)
        self.live_snapshots += 1
        self._survival_points += 10
        if self._previous_aggregate is not None:
            drop = self._previous_aggregate - aggregate
            if drop >= 6:
                cleared = min(4, max(1, round(drop / max(PF_COLS, 1))))
                self.estimated_lines += cleared
                self._clear_points += (0, 1000, 3000, 5000, 8000)[cleared]
        self._previous_aggregate = aggregate
        danger = holes * 35 + (max(heights) if heights else 0) * 4 + bumpiness * 2
        self.score = self._survival_points + self._clear_points - danger
        return self.score

    def result(self) -> dict:
        return {
            "score": self.score,
            "estimated_lines": self.estimated_lines,
            "live_snapshots": self.live_snapshots,
        }


def pick_race_move(
    heights: list[int],
    n: int,
    weights: list[float],
    rng: random.Random,
) -> int:
    """Genome-weighted board policy used by both tournament and champion."""
    if len(weights) != len(default_play_weights()):
        weights = default_play_weights()
    target = min(range(len(heights)), key=lambda c: (heights[c], abs(c - 4))) if heights else 4
    max_h = max(heights) if heights else 0
    scores = [
        weights[0] + (weights[5] if max_h >= 12 else 0.0),  # DOWN
        weights[1] + max(0, 4 - target) * 0.45,             # LEFT
        weights[2] + max(0, target - 5) * 0.45,             # RIGHT
        weights[3] + (0.35 if n % 5 == 0 else 0.0),         # A
        weights[4] + (0.25 if n % 7 == 0 else 0.0),         # B
    ]
    actions = (DOWN, LEFT, RIGHT, A, B)
    m = max(scores)
    exps = [math.exp(s - m) for s in scores]
    pick = rng.random() * sum(exps)
    acc = 0.0
    for action, weight in zip(actions, exps):
        acc += weight
        if pick <= acc:
            return action
    return DOWN


def race_rank_key(result: dict) -> tuple[int, int, int]:
    """Stage always dominates; actual LIVE race score breaks stage ties."""
    return (
        int(result.get("stage") or 0),
        int(result.get("score") or 0),
        int(result.get("final") or 0),
    )


def generation_has_valid_bot(results: list[dict]) -> bool:
    """A transport/load failure must never overwrite a learned genome."""
    return any(not result.get("error") for result in results)


def run_gameplay_race(
    client: PlayClient,
    weights: list[float],
    seconds: float,
    rng: random.Random,
    play_hz: float = 7.0,
) -> dict:
    """Play one LIVE session for a bounded round and return board fitness."""
    tracker = BoardFitness()
    deadline = time.monotonic() + max(1.0, seconds)
    n = 0
    seen_live = False
    while time.monotonic() < deadline:
        code, rgb = client.frame_rgb()
        if code != 200 or not isinstance(rgb, (bytes, bytearray)):
            time.sleep(0.08)
            continue
        rgb_b = bytes(rgb)
        stage = classify_stage(rgb_b)
        if stage != 5:
            if seen_live:
                break
            time.sleep(0.08)
            continue
        seen_live = True
        heights, holes, bumpiness = board_metrics(rgb_b)
        tracker.observe(heights, holes, bumpiness)
        n += 1
        action = pick_race_move(heights, n, weights, rng)
        hold(client, action, 0.10 if action == DOWN else 0.07)
        time.sleep(max(0.0, 1.0 / max(play_hz, 1.0) - 0.07))
    result = tracker.result()
    result["live"] = seen_live
    return result


def read_proc(pid: int) -> dict | None:
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8") as f:
            parts = f.read().split()
        utime = int(parts[13])
        stime = int(parts[14])
        rss_pages = int(parts[23])
        page = os.sysconf("SC_PAGE_SIZE")
        return {"pid": pid, "cpu_ticks": utime + stime, "rss_kib": (rss_pages * page) // 1024}
    except (FileNotFoundError, IndexError, ValueError, OSError):
        return None


def find_play_pids() -> list[int]:
    pids = []
    try:
        for name in Path("/proc").iterdir():
            if not name.name.isdigit():
                continue
            try:
                cmd = (name / "cmdline").read_bytes().replace(b"\x00", b" ").decode(
                    "utf-8", errors="ignore"
                )
            except OSError:
                continue
            if "softgbc-play" in cmd:
                pids.append(int(name.name))
    except OSError:
        pass
    return pids


def host_snapshot(prev_ticks: dict[int, tuple[float, int]]) -> tuple[str, dict]:
    now = time.monotonic()
    hz = os.sysconf("SC_CLK_TCK")
    bits = []
    new_prev: dict[int, tuple[float, int]] = {}
    try:
        load1, load5, load15 = os.getloadavg()
        bits.append(f"load={load1:.2f}/{load5:.2f}/{load15:.2f}")
    except OSError:
        pass
    for pid in find_play_pids():
        info = read_proc(pid)
        if not info:
            continue
        ticks = info["cpu_ticks"]
        cpu_pct = 0.0
        if pid in prev_ticks:
            t0, ticks0 = prev_ticks[pid]
            dt = now - t0
            if dt > 0.05:
                cpu_pct = 100.0 * (ticks - ticks0) / (hz * dt)
        new_prev[pid] = (now, ticks)
        bits.append(f"play[{pid}] cpu={cpu_pct:.1f}% rss={info['rss_kib']}KiB")
    return (" ".join(bits) if bits else "host=n/a", new_prev)


def analyze_http(prev: dict | None, status: dict, debug: dict) -> list[str]:
    flags: list[str] = []
    if not status.get("loaded") and not debug.get("loaded"):
        return ["session_unloaded"]
    frames = status.get("frames")
    if frames is None and isinstance(debug.get("run"), dict):
        frames = debug["run"].get("frames")
    running = status.get("running")
    if running is None and isinstance(debug.get("run"), dict):
        running = debug["run"].get("running")
    cpu = debug.get("cpu") or {}
    pc = cpu.get("pc")
    halted = bool(cpu.get("halted"))

    if prev:
        if running and prev.get("frames") == frames and frames is not None:
            flags.append(f"frame_stall frames={frames}")
        # Tetris DX spends most of LIVE in HALT at $0283 waiting for VBlank —
        # frames advancing with a stable PC/halt is normal, not a hang.
        vblank_wait = pc == 0x0283 and frames is not None and prev.get("frames") is not None and frames > prev["frames"]
        if (
            not vblank_wait
            and frames is not None
            and prev.get("frames") is not None
            and frames > prev["frames"]
            and pc is not None
            and pc == prev.get("pc")
            and (prev.get("pc_same", 0) + 1) >= 8
        ):
            flags.append(f"pc_stuck pc=0x{pc:04X}")
        if (
            not vblank_wait
            and halted
            and prev.get("halted")
            and (prev.get("halt_same", 0) + 1) >= 12
        ):
            flags.append("halted_stuck")
        ie = (debug.get("irq") or {}).get("ie")
        if ie is not None and (ie & 1) and prev.get("if_zero_streak", 0) >= 40:
            flags.append("vblank_if_never_seen")
    return flags


def analyze_gdb(gdb_snap: dict, prev_gdb: dict | None) -> list[str]:
    flags: list[str] = []
    regs = gdb_snap.get("regs") or {}
    mem = gdb_snap.get("mem_at_pc") or []
    pc = regs.get("pc")
    sp = regs.get("sp")
    if pc is None:
        return ["gdb_no_pc"]
    if sp is not None and (sp < 0xC000 or sp > 0xFFFE):
        # Not always wrong (stack can be elsewhere) — flag extreme cases.
        if sp < 0x8000:
            flags.append(f"gdb_sp_suspicious sp=0x{sp:04X}")
    if mem:
        op = mem[0]
        # Illegal / unused SM83 opcodes often seen when PC wanders into garbage.
        illegal = {0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD}
        if op in illegal:
            flags.append(f"gdb_illegal_opcode op=0x{op:02X} pc=0x{pc:04X}")
    if prev_gdb and prev_gdb.get("pc") == pc and prev_gdb.get("mem") == mem:
        if prev_gdb.get("same", 0) + 1 >= 3:
            flags.append(f"gdb_pc_frozen pc=0x{pc:04X} mem={mem}")
    return flags


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="http://127.0.0.1:8000")
    ap.add_argument("--session", default="softgbcBot1")
    ap.add_argument("--rom", default=TETRIS)
    ap.add_argument("--seconds", type=float, default=0.0)
    ap.add_argument("--play-hz", type=float, default=6.0)
    ap.add_argument("--vision-every", type=float, default=0.75, help="how often to pull frame.rgb")
    ap.add_argument("--probe-every", type=float, default=3.0)
    ap.add_argument("--gdb-every", type=float, default=8.0)
    ap.add_argument("--log", type=Path, default=DEFAULT_LOG)
    ap.add_argument("--no-reload", action="store_true")
    ap.add_argument("--no-menu", action="store_true")
    ap.add_argument(
        "--darwin",
        action="store_true",
        help="evolve menu genomes across bots, then play as champion/session",
    )
    ap.add_argument("--bots", type=int, default=4, help="Darwin population size")
    ap.add_argument("--generations", type=int, default=3)
    ap.add_argument("--gen-seconds", type=float, default=28.0)
    ap.add_argument(
        "--race-seconds",
        type=float,
        default=45.0,
        help="LIVE gameplay round length per generation",
    )
    args = ap.parse_args()

    anomalies: list[str] = []
    play_n = probes = gdb_n = http_errors = 0
    prev_core: dict | None = None
    prev_gdb: dict | None = None
    prev_ticks: dict[int, tuple[float, int]] = {}
    play_dt = 1.0 / max(args.play_hz, 1.0)
    cached_gdb_port = 0
    last_heights: list[int] | None = None
    genome = load_genome()

    with args.log.open("w", encoding="utf-8") as fp:
        log_line(
            fp,
            f"play_agent start session={args.session} play_hz={args.play_hz} "
            f"gdb_every={args.gdb_every}s darwin={args.darwin}",
        )
        if args.darwin:
            genome = darwin_evolve(
                args.base,
                bots=max(2, args.bots),
                generations=max(1, args.generations),
                gen_seconds=args.gen_seconds,
                race_seconds=args.race_seconds,
                fp=fp,
            )
            # Promote champion session into continuous play when it reached LIVE.
            if genome.get("champion") and int(genome.get("fitness") or 0) >= 5:
                args.session = str(genome["champion"])
                args.no_reload = True
                args.no_menu = True
                log_line(fp, f"darwin: promote champion session={args.session}")
            else:
                log_line(
                    fp,
                    f"darwin: no LIVE champ — continue on {args.session} with best genome",
                )

        client = PlayClient(args.base, args.session)
        if not args.no_reload:
            code, load = client.load_builtin(args.rom)
            log_line(fp, f"load http={code} body={json.dumps(load)[:260]}")
            if code != 200 or not (isinstance(load, dict) and load.get("ok")):
                log_line(fp, "FATAL: load failed")
                return 1
            if not args.no_menu:
                enter_tetris(client, fp, genome=genome)
        else:
            log_line(fp, f"join existing session={args.session}")

        deadline = None if args.seconds <= 0 else time.monotonic() + args.seconds
        next_probe = time.monotonic()
        next_play = time.monotonic()
        next_vision = time.monotonic()
        next_gdb = time.monotonic() + 3.0
        seen_live = False
        play_rng = random.Random(0xB07 ^ int(genome.get("generation") or 0))
        play_weights = list(genome.get("play_weights") or default_play_weights())

        while deadline is None or time.monotonic() < deadline:
            try:
                now = time.monotonic()

                if now >= next_vision:
                    next_vision = now + args.vision_every
                    code, rgb = client.frame_rgb()
                    if code == 200 and isinstance(rgb, (bytes, bytearray)) and len(rgb) >= SCREEN_W * SCREEN_H * 3:
                        rgb_b = bytes(rgb)
                        st = classify_stage(rgb_b)
                        if st == 5:
                            seen_live = True
                        # After a real Marathon start, a full well looks like
                        # attract DEMO — never Start-exit once LIVE was seen.
                        if not seen_live and exit_demo_if_needed(client, fp, rgb_b):
                            last_heights = None
                        elif st == 5:
                            last_heights = column_heights(rgb_b)
                        elif st in (3, 4):
                            # Mode/level — confirm into play.
                            tap(client, A, 0.28, 0.5)
                            last_heights = None
                        elif st < 5 and play_n > 0 and play_n % 48 == 0:
                            log_line(fp, f"menu: stuck at {STAGE_NAMES.get(st, st)} — re-drive")
                            run_menu_genome(
                                client,
                                list(genome.get("weights") or default_genome()),
                                seconds=20.0,
                                rng=random.Random(play_n),
                            )
                            last_heights = None
                        else:
                            last_heights = None
                    elif code == 503 or code == 0:
                        pass
                    else:
                        http_errors += 1

                if now >= next_play:
                    next_play = now + play_dt
                    play_n += 1
                    heights = last_heights
                    if heights is not None:
                        btn = pick_race_move(heights, play_n, play_weights, play_rng)
                    else:
                        btn = (DOWN, DOWN, A, LEFT, DOWN, RIGHT, B, DOWN)[play_n % 8]
                    # Hold longer so the game actually samples the key.
                    hold_s = 0.11 if btn == DOWN else 0.08
                    if heights and max(heights) >= 12 and btn in (LEFT, RIGHT, A):
                        hold(client, DOWN, 0.08)
                    hold(client, btn, hold_s)
                    if play_n % 24 == 0:
                        mh = max(heights) if heights else -1
                        log_line(fp, f"play n={play_n} btn={btn} maxH={mh}")

                if now >= next_probe:
                    next_probe = now + args.probe_every
                    probes += 1
                    sc, status = client.status()
                    dc, debug = client.debug()
                    if sc >= 500 or sc == 0 or dc >= 500 or dc == 0:
                        http_errors += 1
                        log_line(fp, f"WARN http status={sc} debug={dc}")
                        anomalies.append(f"http_error status={sc} debug={dc}")
                    else:
                        status = status if isinstance(status, dict) else {}
                        debug = debug if isinstance(debug, dict) else {}
                        frames = status.get("frames")
                        if frames is None and isinstance(debug.get("run"), dict):
                            frames = debug["run"].get("frames")
                        cpu = debug.get("cpu") or {}
                        ppu = debug.get("ppu") or {}
                        irq = debug.get("irq") or {}
                        gdb_meta = debug.get("gdb") or {}
                        host_line, prev_ticks = host_snapshot(prev_ticks)
                        log_line(
                            fp,
                            "probe "
                            f"frames={frames} pc={_hex(cpu.get('pc'))} "
                            f"ly={ppu.get('ly')} ime={cpu.get('ime')} "
                            f"halted={cpu.get('halted')} if={_hex(irq.get('if'))} "
                            f"gdb_port={status.get('gdb_port') or gdb_meta.get('port')} "
                            f"gdb_halted={gdb_meta.get('halted')} | {host_line}",
                        )
                        for f in analyze_http(prev_core, status, debug):
                            log_line(fp, f"ANOMALY {f}")
                            anomalies.append(f)
                        if_zero = (irq.get("if") or 0) == 0
                        prev_core = {
                            "frames": frames,
                            "pc": cpu.get("pc"),
                            "halted": cpu.get("halted"),
                            "pc_same": (
                                (prev_core or {}).get("pc_same", 0) + 1
                                if prev_core and prev_core.get("pc") == cpu.get("pc")
                                else 0
                            ),
                            "halt_same": (
                                (prev_core or {}).get("halt_same", 0) + 1
                                if prev_core and prev_core.get("halted") and cpu.get("halted")
                                else 0
                            ),
                            "if_zero_streak": (
                                (prev_core or {}).get("if_zero_streak", 0) + 1
                                if if_zero
                                else 0
                            ),
                        }

                if now >= next_gdb:
                    next_gdb = now + args.gdb_every
                    gdb_n += 1
                    gdb_port = cached_gdb_port or client.gdb_port_for_self()
                    if gdb_port:
                        cached_gdb_port = gdb_port
                    if not gdb_port:
                        log_line(fp, "WARN gdb_port missing — is softgbc-play new enough?")
                    else:
                        try:
                            snap = GdbClient("127.0.0.1", gdb_port).sample()
                            regs = snap["regs"]
                            mem = snap["mem_at_pc"]
                            log_line(
                                fp,
                                "gdb "
                                f"pc={_hex(regs.get('pc'))} sp={_hex(regs.get('sp'))} "
                                f"af={_hex(regs.get('af'))} bc={_hex(regs.get('bc'))} "
                                f"de={_hex(regs.get('de'))} hl={_hex(regs.get('hl'))} "
                                f"ime={regs.get('ime')} mem@pc={mem}",
                            )
                            for f in analyze_gdb(snap, prev_gdb):
                                log_line(fp, f"ANOMALY {f}")
                                anomalies.append(f)
                            prev_gdb = {
                                "pc": regs.get("pc"),
                                "mem": mem,
                                "same": (
                                    (prev_gdb or {}).get("same", 0) + 1
                                    if prev_gdb
                                    and prev_gdb.get("pc") == regs.get("pc")
                                    and prev_gdb.get("mem") == mem
                                    else 0
                                ),
                            }
                        except Exception as e:
                            log_line(fp, f"WARN gdb_sample failed: {e}")
                            anomalies.append(f"gdb_sample_error {e}")

                time.sleep(0.004)
            except Exception as e:
                http_errors += 1
                log_line(fp, f"WARN loop recovered: {type(e).__name__}: {e}")
                time.sleep(0.5)

        uniq = sorted(set(anomalies))
        log_line(
            fp,
            f"done plays={play_n} probes={probes} gdb={gdb_n} "
            f"http_errors={http_errors} anomalies={len(uniq)}",
        )
        for a in uniq:
            log_line(fp, f"summary {a}")
        if not uniq:
            log_line(fp, "summary no anomalies flagged")
    return 0 if http_errors == 0 else 2


def _hex(v) -> str:
    if v is None:
        return "-"
    try:
        return f"0x{int(v):04X}"
    except (TypeError, ValueError):
        return str(v)


if __name__ == "__main__":
    raise SystemExit(main())
