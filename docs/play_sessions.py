"""Per-session SoftGBC play — one softgbc-play process per shareable id.

Clients pass ``X-Play-Session`` (or ``?s=`` / cookie). Same id → same emu;
different ids → isolated instances. Share the Environment URL with ``?s=…``
so others can watch (and play) the same session.
"""

from __future__ import annotations

import json
import re
import secrets
import socket
import subprocess
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable
from urllib.parse import parse_qs, urlparse

COOKIE_NAME = "softgbc_play"
HEADER_NAME = "X-Play-Session"
MAX_SESSIONS = 12
IDLE_TTL_SEC = 30 * 60
# URL-safe short ids: letters, digits, _- (token_urlsafe alphabet)
SID_RE = re.compile(r"^[A-Za-z0-9_-]{6,64}$")


@dataclass
class PlaySession:
    sid: str
    port: int
    gdb_port: int = 0
    proc: subprocess.Popen | None = None
    last_used: float = field(default_factory=time.time)
    title: str = ""
    frames: int = 0
    running: bool = False

    def touch(self) -> None:
        self.last_used = time.time()

    def alive(self) -> bool:
        return self.proc is not None and self.proc.poll() is None


def new_session_id() -> str:
    # ~12 chars URL-safe — short enough to read aloud / paste.
    return secrets.token_urlsafe(9)


def valid_session_id(sid: str | None) -> str | None:
    if not sid:
        return None
    s = sid.strip()
    if not SID_RE.match(s):
        return None
    return s


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def parse_cookie_header(header: str | None, name: str = COOKIE_NAME) -> str | None:
    if not header:
        return None
    for part in header.split(";"):
        part = part.strip()
        if not part or "=" not in part:
            continue
        k, v = part.split("=", 1)
        if k.strip() == name:
            return valid_session_id(v.strip())
    return None


def cookie_set_header(sid: str, name: str = COOKIE_NAME) -> str:
    return f"{name}={sid}; Path=/; SameSite=Lax; Max-Age={IDLE_TTL_SEC}"


def resolve_session_id(
    *,
    header: str | None = None,
    cookie: str | None = None,
    query_s: str | None = None,
) -> str | None:
    """Prefer explicit header, then ``?s=``, then cookie."""
    for cand in (header, query_s, cookie):
        ok = valid_session_id(cand)
        if ok:
            return ok
    return None


def query_s_from_path(path_with_query: str) -> str | None:
    qs = parse_qs(urlparse(path_with_query).query)
    vals = qs.get("s") or []
    return valid_session_id(vals[0]) if vals else None


class PlaySessionManager:
    """Thread-safe map of session id → softgbc-play child."""

    def __init__(
        self,
        play_bin: Path,
        *,
        max_sessions: int = MAX_SESSIONS,
        idle_ttl_sec: float = IDLE_TTL_SEC,
        popen: Callable[..., subprocess.Popen] | None = None,
        port_picker: Callable[[], int] | None = None,
        probe: Callable[[int], bool] | None = None,
    ) -> None:
        self.play_bin = Path(play_bin)
        self.max_sessions = max_sessions
        self.idle_ttl_sec = idle_ttl_sec
        self._popen = popen or subprocess.Popen
        self._port_picker = port_picker or pick_free_port
        self._probe = probe or self._default_probe
        self._lock = threading.Lock()
        self._sessions: dict[str, PlaySession] = {}

    def session_count(self) -> int:
        with self._lock:
            return len(self._sessions)

    def ports_for(self, sid: str) -> int | None:
        with self._lock:
            s = self._sessions.get(sid)
            return s.port if s and s.alive() else None

    def update_meta(
        self,
        sid: str,
        *,
        title: str | None = None,
        frames: int | None = None,
        running: bool | None = None,
    ) -> None:
        sid = valid_session_id(sid)
        if not sid:
            return
        with self._lock:
            s = self._sessions.get(sid)
            if not s:
                return
            s.touch()
            if title is not None:
                s.title = title
            if frames is not None:
                s.frames = int(frames)
            if running is not None:
                s.running = bool(running)

    def list_live(self) -> list[dict]:
        """Public snapshot of alive sessions (for the Environment sidebar)."""
        now = time.time()
        with self._lock:
            self._reap_locked()
            out = []
            for s in sorted(
                self._sessions.values(), key=lambda x: x.last_used, reverse=True
            ):
                if not s.alive():
                    continue
                out.append(
                    {
                        "id": s.sid,
                        "title": s.title or "(no ROM)",
                        "frames": s.frames,
                        "running": s.running,
                        "idle_sec": int(max(0, now - s.last_used)),
                        "gdb_port": s.gdb_port,
                    }
                )
            return out

    def ensure(self, sid: str) -> tuple[PlaySession | None, str | None]:
        """Ensure a live process for ``sid``. Returns (session, error)."""
        sid = valid_session_id(sid)
        if not sid:
            return None, "invalid session id"

        with self._lock:
            self._reap_locked()
            existing = self._sessions.get(sid)
            if existing and existing.alive():
                existing.touch()
                return existing, None
            if existing:
                self._kill_locked(existing)
                del self._sessions[sid]

            if len(self._sessions) >= self.max_sessions:
                oldest = min(self._sessions.values(), key=lambda s: s.last_used)
                self._kill_locked(oldest)
                del self._sessions[oldest.sid]

            err = self._spawn_locked(sid)
            if err:
                return None, err
            return self._sessions[sid], None

    def stop(self, sid: str | None) -> None:
        sid = valid_session_id(sid)
        if not sid:
            return
        with self._lock:
            sess = self._sessions.pop(sid, None)
            if sess:
                self._kill_locked(sess)

    def shutdown_all(self) -> None:
        with self._lock:
            for sess in list(self._sessions.values()):
                self._kill_locked(sess)
            self._sessions.clear()

    def _spawn_locked(self, sid: str) -> str | None:
        if not self.play_bin.is_file():
            return f"softgbc-play missing: {self.play_bin}"
        port = self._port_picker()
        gdb_port = self._port_picker()
        try:
            proc = self._popen(
                [
                    str(self.play_bin),
                    "--port",
                    str(port),
                    "--gdb-port",
                    str(gdb_port),
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
            )
        except OSError as e:
            return str(e)

        deadline = time.time() + 3.0
        while time.time() < deadline:
            if proc.poll() is not None:
                err = b""
                if proc.stderr:
                    err = proc.stderr.read() or b""
                return err.decode("utf-8", "replace").strip() or "softgbc-play exited"
            if self._probe(port):
                break
            time.sleep(0.05)

        # Prefer gdb_port reported by /status (binds may remap).
        reported = self._status_gdb_port(port) or gdb_port
        self._sessions[sid] = PlaySession(
            sid=sid, port=port, gdb_port=reported, proc=proc
        )
        return None

    @staticmethod
    def _status_gdb_port(http_port: int) -> int:
        import urllib.request

        try:
            with urllib.request.urlopen(
                f"http://127.0.0.1:{http_port}/status", timeout=0.5
            ) as resp:
                obj = json.loads(resp.read().decode("utf-8"))
                return int(obj.get("gdb_port") or 0)
        except Exception:
            return 0

    @staticmethod
    def _default_probe(port: int) -> bool:
        import urllib.request

        try:
            with urllib.request.urlopen(
                f"http://127.0.0.1:{port}/status", timeout=0.2
            ) as resp:
                return resp.status == 200
        except Exception:
            return False

    def _reap_locked(self) -> None:
        now = time.time()
        dead = [
            sid
            for sid, s in self._sessions.items()
            if (not s.alive()) or (now - s.last_used > self.idle_ttl_sec)
        ]
        for sid in dead:
            self._kill_locked(self._sessions.pop(sid))

    @staticmethod
    def _kill_locked(sess: PlaySession) -> None:
        proc = sess.proc
        if proc is None:
            return
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                proc.kill()
        sess.proc = None


def play_http_to(
    port: int,
    method: str,
    path: str,
    body: bytes | None = None,
    timeout: float = 30.0,
) -> tuple[int, bytes, str]:
    import urllib.error
    import urllib.request

    url = f"http://127.0.0.1:{port}{path}"
    data = body if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    if method == "POST":
        if path == "/input":
            req.add_header("Content-Type", "application/json")
        elif path == "/load":
            req.add_header("Content-Type", "application/octet-stream")
        if data is not None:
            req.add_header("Content-Length", str(len(data)))
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            ctype = resp.headers.get("Content-Type", "application/octet-stream")
            return resp.status, resp.read(), ctype
    except urllib.error.HTTPError as e:
        ctype = (
            e.headers.get("Content-Type", "application/octet-stream")
            if e.headers
            else "text/plain"
        )
        return e.code, e.read(), ctype
    except Exception as e:
        return 502, json.dumps({"error": str(e)}).encode("utf-8"), "application/json"
