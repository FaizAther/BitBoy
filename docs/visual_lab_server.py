#!/usr/bin/env python3
"""Serve docs/site plus SoftGBC visual-lab API (play → PNG + expected CRC).

With --watch: rebuild site / softgbc-lab on source changes and bump /api/dev/build
so the browser live_reload.js refreshes.
"""

from __future__ import annotations

import argparse
import json
import mimetypes
import os
import socket
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlparse

from play_sessions import (
    COOKIE_NAME,
    HEADER_NAME,
    PlaySessionManager,
    cookie_set_header,
    new_session_id,
    parse_cookie_header,
    play_http_to,
    query_s_from_path,
    resolve_session_id,
)

DOCS = Path(__file__).resolve().parent
SOFTGBC = DOCS.parent / "SoftGBC"
DEFAULT_SITE = DOCS / "site"
DEFAULT_LAB = SOFTGBC / "target" / "debug" / "softgbc-lab"
DEFAULT_PLAY = SOFTGBC / "target" / "release" / "softgbc-play"
# Fall back to debug if release has not been built yet.
if not DEFAULT_PLAY.is_file():
    DEFAULT_PLAY = SOFTGBC / "target" / "debug" / "softgbc-play"
FIXTURES = SOFTGBC / "dmg-ppu" / "fixtures"
ROMS_DIR = SOFTGBC / "cartridge" / "roms"

# Shared with the watcher thread.
BUILD_STATE = {"id": 1, "msg": "boot", "busy": False}
BUILD_LOCK = threading.Lock()

# One softgbc-play process per shareable session id (?s= / X-Play-Session).
PLAY_MGR: PlaySessionManager | None = None
# Set in main() so /api/play/reach can advertise a Mac/LAN URL.
SERVER_PORT = 8000
SERVER_BIND = "0.0.0.0"


def _default_route_ip() -> str | None:
    """IPv4 used for the default route (Wi‑Fi/LAN the Mac can usually reach)."""
    try:
        out = subprocess.check_output(
            ["ip", "-4", "route", "show", "default"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
        # e.g. default via 192.168.50.1 dev wlan0 proto dhcp src 192.168.50.160
        for line in out.splitlines():
            parts = line.split()
            if "src" in parts:
                return parts[parts.index("src") + 1]
    except Exception:
        pass
    return None


def _lan_ips() -> list[str]:
    """Best-effort non-loopback IPv4 addresses for Mac/LAN reach tips."""
    preferred = _default_route_ip()
    found: list[str] = []
    try:
        out = subprocess.check_output(
            ["hostname", "-I"], text=True, stderr=subprocess.DEVNULL
        )
        for tok in out.split():
            if tok and not tok.startswith("127.") and ":" not in tok:
                found.append(tok)
    except Exception:
        pass
    if not found:
        try:
            lan = socket.gethostbyname(socket.gethostname())
            if lan and not lan.startswith("127."):
                found.append(lan)
        except Exception:
            pass
    uniq = list(dict.fromkeys(found))
    if preferred and preferred in uniq:
        uniq.remove(preferred)
        uniq.insert(0, preferred)
    elif preferred:
        uniq.insert(0, preferred)
    # Deprioritize docker/libvirt/VPN-ish ranges after the default-route IP.
    def _rank(ip: str) -> tuple[int, str]:
        if preferred and ip == preferred:
            return (0, ip)
        if ip.startswith("192.168.122."):  # virbr0
            return (4, ip)
        if ip.startswith("172.17.") or ip.startswith("172.18."):
            return (5, ip)
        if ip.startswith("10."):
            return (3, ip)
        if ip.startswith("192.168."):
            return (1, ip)
        if ip.startswith("172."):
            return (2, ip)
        return (6, ip)

    return sorted(uniq, key=_rank)

# Guard against browser tabs stacking hundreds of parallel frame polls
# (old play.js created a new Image every 50ms without waiting → DoS).
_FRAME_INFLIGHT: set[str] = set()
_FRAME_INFLIGHT_LOCK = threading.Lock()
_PLAY_PROXY_SEM = threading.Semaphore(4)
# Hard cap on concurrent handler threads (ThreadingHTTPServer is otherwise unbounded).
_MAX_HTTP_WORKERS = 24


class LimitedThreadingHTTPServer(ThreadingHTTPServer):
    """Drop connections when too many handlers are already running.

    Without this, stacked frame polls open thousands of threads and freeze
    the host (and any machine driving the browser).
    """

    daemon_threads = True
    allow_reuse_address = True
    request_queue_size = 32

    def __init__(self, *args, max_workers: int = _MAX_HTTP_WORKERS, **kwargs):
        self._workers = threading.Semaphore(max_workers)
        super().__init__(*args, **kwargs)

    def process_request(self, request, client_address):
        if not self._workers.acquire(blocking=False):
            try:
                request.shutdown(socket.SHUT_RDWR)
            except Exception:
                pass
            try:
                request.close()
            except Exception:
                pass
            return

        t = threading.Thread(
            target=self._process_request_thread,
            args=(request, client_address),
            daemon=True,
        )
        t.start()

    def _process_request_thread(self, request, client_address):
        try:
            self.finish_request(request, client_address)
        except Exception:
            self.handle_error(request, client_address)
        finally:
            try:
                self.shutdown_request(request)
            finally:
                self._workers.release()


def bump_build(msg: str) -> int:
    with BUILD_LOCK:
        BUILD_STATE["id"] = int(BUILD_STATE["id"]) + 1
        BUILD_STATE["msg"] = msg
        return int(BUILD_STATE["id"])


def get_play_mgr() -> PlaySessionManager:
    global PLAY_MGR
    if PLAY_MGR is None:
        PLAY_MGR = PlaySessionManager(DEFAULT_PLAY)
    return PLAY_MGR


def list_roms() -> list[str]:
    if not ROMS_DIR.is_dir():
        return []
    out = []
    for p in sorted(ROMS_DIR.iterdir()):
        if p.is_file() and p.suffix.lower() in {".gb", ".gbc", ".bin"}:
            out.append(p.name)
    return out


def resolve_rom_name(name: str) -> Path | None:
    """Safe join into ROMS_DIR (basename only)."""
    base = Path(name).name
    if not base or base != name or ".." in name:
        return None
    path = (ROMS_DIR / base).resolve()
    try:
        path.relative_to(ROMS_DIR.resolve())
    except ValueError:
        return None
    if path.is_file():
        return path
    return None


class LabHandler(BaseHTTPRequestHandler):
    site_root: Path = DEFAULT_SITE
    lab_bin: Path = DEFAULT_LAB
    play_bin: Path = DEFAULT_PLAY

    def log_message(self, fmt: str, *args) -> None:
        # Frame polls are ~60/s; logging them saturates the process and disk.
        try:
            msg = fmt % args
        except Exception:
            msg = fmt
        if "/api/play/frame." in msg or "/api/dev/build" in msg:
            return
        sys.stderr.write("%s - %s\n" % (self.address_string(), msg))

    def _send(
        self,
        code: int,
        body: bytes,
        content_type: str,
        *,
        extra_headers: list[tuple[str, str]] | None = None,
    ) -> None:
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        if extra_headers:
            for k, v in extra_headers:
                self.send_header(k, v)
        self.end_headers()
        self.wfile.write(body)

    def _json(
        self,
        code: int,
        obj: object,
        *,
        extra_headers: list[tuple[str, str]] | None = None,
    ) -> None:
        data = json.dumps(obj).encode("utf-8")
        self._send(
            code, data, "application/json; charset=utf-8", extra_headers=extra_headers
        )

    def _read_body(self, max_bytes: int = 8 * 1024 * 1024) -> bytes:
        n = int(self.headers.get("Content-Length", "0") or "0")
        if n <= 0:
            return b""
        if n > max_bytes:
            raise ValueError("body too large")
        return self.rfile.read(n)

    def _play_sid(self) -> str | None:
        return resolve_session_id(
            header=self.headers.get(HEADER_NAME),
            query_s=query_s_from_path(self.path),
            cookie=parse_cookie_header(self.headers.get("Cookie")),
        )

    def _play_proxy(
        self,
        method: str,
        play_path: str,
        body: bytes | None = None,
        *,
        timeout: float = 30.0,
        stop_after: bool = False,
        require_sid: bool = True,
    ) -> None:
        sid = self._play_sid()
        if not sid:
            if require_sid:
                self._json(
                    400,
                    {
                        "error": "missing session id — open Environment with ?s=… or send X-Play-Session",
                    },
                )
                return
            sid = new_session_id()

        extras = [
            ("Set-Cookie", cookie_set_header(sid)),
            ("X-Play-Session", sid),
        ]
        if stop_after:
            port = get_play_mgr().ports_for(sid)
            if port:
                play_http_to(port, method, play_path, body, timeout=timeout)
            get_play_mgr().stop(sid)
            self._json(200, {"ok": True, "session": sid}, extra_headers=extras)
            return

        is_frame = play_path.startswith("/frame.")

        # Frame/status polls must NEVER spawn an emu. Only /load does that.
        # Spawning on poll + stacked Image() requests froze the machine.
        if is_frame or play_path == "/status":
            port = get_play_mgr().ports_for(sid)
            if not port:
                if is_frame:
                    self.send_response(204)
                    self.send_header("Cache-Control", "no-store")
                    self.send_header("X-Play-Session", sid)
                    self.end_headers()
                else:
                    self._json(
                        200,
                        {
                            "loaded": False,
                            "running": False,
                            "frames": 0,
                            "title": "",
                            "session": sid,
                        },
                        extra_headers=extras,
                    )
                return
            if is_frame:
                with _FRAME_INFLIGHT_LOCK:
                    if sid in _FRAME_INFLIGHT:
                        self.send_response(204)
                        self.send_header("Cache-Control", "no-store")
                        self.send_header("X-Play-Session", sid)
                        self.end_headers()
                        return
                    _FRAME_INFLIGHT.add(sid)
            if not _PLAY_PROXY_SEM.acquire(blocking=False):
                if is_frame:
                    with _FRAME_INFLIGHT_LOCK:
                        _FRAME_INFLIGHT.discard(sid)
                    self.send_response(204)
                    self.send_header("Cache-Control", "no-store")
                    self.end_headers()
                else:
                    self._json(
                        503, {"error": "play busy", "session": sid}, extra_headers=extras
                    )
                return
            try:
                code, resp, ctype = play_http_to(
                    port, method, play_path, body, timeout=1.5 if is_frame else 5.0
                )
                if is_frame:
                    # Watching counts as activity so the session stays listed.
                    get_play_mgr().update_meta(sid)
                if ctype.startswith("application/json") and resp.startswith(b"{"):
                    try:
                        obj = json.loads(resp)
                        if isinstance(obj, dict):
                            obj["session"] = sid
                            get_play_mgr().update_meta(
                                sid,
                                title=obj.get("title"),
                                frames=obj.get("frames"),
                                running=obj.get("running"),
                            )
                            resp = json.dumps(obj).encode("utf-8")
                    except json.JSONDecodeError:
                        pass
                self._send(code, resp, ctype, extra_headers=extras)
            finally:
                _PLAY_PROXY_SEM.release()
                if is_frame:
                    with _FRAME_INFLIGHT_LOCK:
                        _FRAME_INFLIGHT.discard(sid)
            return

        if not _PLAY_PROXY_SEM.acquire(blocking=True, timeout=10):
            self._json(503, {"error": "play busy", "session": sid}, extra_headers=extras)
            return

        try:
            sess, err = get_play_mgr().ensure(sid)
            if err or sess is None:
                self._json(
                    503,
                    {"error": err or "no session", "session": sid},
                    extra_headers=extras,
                )
                return
            code, resp, ctype = play_http_to(
                sess.port, method, play_path, body, timeout=timeout
            )
            if ctype.startswith("application/json") and resp.startswith(b"{"):
                try:
                    obj = json.loads(resp)
                    if isinstance(obj, dict):
                        obj["session"] = sid
                        if play_path == "/load" and obj.get("ok"):
                            get_play_mgr().update_meta(
                                sid,
                                title=obj.get("title") or "",
                                frames=0,
                                running=True,
                            )
                        resp = json.dumps(obj).encode("utf-8")
                except json.JSONDecodeError:
                    pass
            self._send(code, resp, ctype, extra_headers=extras)
        finally:
            _PLAY_PROXY_SEM.release()

    def _run_lab(self, *args: str) -> tuple[int, str, str]:
        if not self.lab_bin.is_file():
            return 127, "", f"lab binary missing: {self.lab_bin}"
        try:
            p = subprocess.run(
                [str(self.lab_bin), *args],
                capture_output=True,
                text=True,
                timeout=60,
                check=False,
            )
            return p.returncode, p.stdout, p.stderr
        except subprocess.TimeoutExpired:
            return 124, "", "lab timed out"
        except OSError as e:
            return 126, "", str(e)

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        path = unquote(parsed.path)

        if path == "/api/dev/build":
            with BUILD_LOCK:
                self._json(
                    200,
                    {
                        "id": BUILD_STATE["id"],
                        "msg": BUILD_STATE["msg"],
                        "busy": BUILD_STATE["busy"],
                    },
                )
            return

        if path == "/api/visual-tests":
            code, out, err = self._run_lab("list")
            if code != 0:
                self._json(500, {"error": err or out or f"exit {code}"})
                return
            try:
                self._json(200, json.loads(out))
            except json.JSONDecodeError:
                self._json(500, {"error": "invalid lab JSON", "raw": out})
            return

        if path == "/api/play/roms":
            self._json(200, {"roms": list_roms(), "dir": str(ROMS_DIR)})
            return

        if path == "/api/play/sessions":
            self._json(200, {"sessions": get_play_mgr().list_live()})
            return

        if path == "/api/play/reach":
            ips = _lan_ips()
            lan = ips[0] if ips else None
            base = f"http://{lan}:{SERVER_PORT}" if lan else None
            self._json(
                200,
                {
                    "port": SERVER_PORT,
                    "bind": SERVER_BIND,
                    "lan_ips": ips,
                    "lan_base": base,
                    "ports_hint": (
                        f"Forward port {SERVER_PORT} in Cursor Ports, then open "
                        f"http://127.0.0.1:{SERVER_PORT}/ on your Mac."
                    ),
                },
            )
            return

        if path == "/api/play/status":
            self._play_proxy("GET", "/status")
            return

        if path == "/api/play/frame.png":
            self._play_proxy("GET", "/frame.png")
            return

        if path == "/api/play/frame.rgb":
            self._play_proxy("GET", "/frame.rgb")
            return

        if path.startswith("/api/fixtures/"):
            name = Path(path.removeprefix("/api/fixtures/")).name
            if not name.endswith(".expected.txt"):
                self._json(400, {"error": "bad fixture name"})
                return
            fp = FIXTURES / name
            if not fp.is_file():
                self._json(404, {"error": "fixture not found"})
                return
            text = fp.read_text(encoding="utf-8")
            self._send(200, text.encode("utf-8"), "text/plain; charset=utf-8")
            return

        rel = path.lstrip("/") or "index.html"
        if ".." in Path(rel).parts:
            self._send(400, b"bad path", "text/plain")
            return
        file_path = (self.site_root / rel).resolve()
        try:
            file_path.relative_to(self.site_root.resolve())
        except ValueError:
            self._send(400, b"bad path", "text/plain")
            return
        if file_path.is_dir():
            file_path = file_path / "index.html"
        if not file_path.is_file():
            self._send(404, b"not found", "text/plain")
            return
        ctype = mimetypes.guess_type(str(file_path))[0] or "application/octet-stream"
        self._send(200, file_path.read_bytes(), ctype)

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        path = unquote(parsed.path)

        if path == "/api/play/load":
            try:
                body = self._read_body()
            except ValueError as e:
                self._json(400, {"error": str(e)})
                return
            self._play_proxy("POST", "/load", body, timeout=60.0)
            return

        if path == "/api/play/load/builtin":
            try:
                raw = self._read_body(64 * 1024)
                obj = json.loads(raw.decode("utf-8") or "{}")
            except (ValueError, json.JSONDecodeError) as e:
                self._json(400, {"error": str(e)})
                return
            name = str(obj.get("name") or "")
            rom_path = resolve_rom_name(name)
            if rom_path is None:
                self._json(404, {"error": "rom not found", "name": name})
                return
            try:
                data = rom_path.read_bytes()
            except OSError as e:
                self._json(500, {"error": str(e)})
                return
            self._play_proxy("POST", "/load", data, timeout=60.0)
            return

        if path == "/api/play/input":
            try:
                body = self._read_body(4096)
            except ValueError as e:
                self._json(400, {"error": str(e)})
                return
            self._play_proxy("POST", "/input", body)
            return

        if path == "/api/play/stop":
            self._play_proxy("POST", "/stop", b"", stop_after=True)
            return

        prefix = "/api/visual-tests/"
        if not path.startswith(prefix) or not path.endswith("/run"):
            self._json(404, {"error": "not found"})
            return
        test_id = path[len(prefix) : -len("/run")].strip("/")
        if not test_id or "/" in test_id or ".." in test_id:
            self._json(400, {"error": "bad test id"})
            return
        code, out, err = self._run_lab("run", test_id)
        if code != 0:
            self._json(500, {"error": err or out or f"exit {code}"})
            return
        try:
            self._json(200, json.loads(out))
        except json.JSONDecodeError:
            self._json(500, {"error": "invalid lab JSON", "raw": out})


def _collect_mtimes(paths: list[Path]) -> dict[str, float]:
    out: dict[str, float] = {}
    for p in paths:
        try:
            if p.is_file():
                out[str(p)] = p.stat().st_mtime
            elif p.is_dir():
                for root, dirs, files in os.walk(p):
                    dirs[:] = [
                        d
                        for d in dirs
                        if d not in {".git", "target", "node_modules", ".venv", "site"}
                    ]
                    for name in files:
                        if name.endswith(
                            (
                                ".py",
                                ".js",
                                ".css",
                                ".md",
                                ".txt",
                                ".html",
                                ".rs",
                                ".toml",
                                ".sh",
                            )
                        ):
                            fp = Path(root) / name
                            try:
                                out[str(fp)] = fp.stat().st_mtime
                            except OSError:
                                pass
        except OSError:
            pass
    return out


def _watch_loop(lab_bin: Path) -> None:
    # Intentionally omit visual_lab_server.py (needs process restart) and site/
    # (output of rebuild). Debounce so one edit → one rebuild → one browser refresh.
    docs_roots = [
        DOCS / "assets",
        DOCS / "build_site.py",
        DOCS / "serve.sh",
        DOCS / "README.md",
        DOCS / "REFERENCES.md",
        DOCS / "datasheets" / "README.md",
        SOFTGBC / "README.md",
        SOFTGBC / "dmg-ppu" / "README.md",
        SOFTGBC / "dmg-ppu" / "fixtures",
        SOFTGBC / "peripherals" / "README.md",
        SOFTGBC / "peripherals" / "src",
        SOFTGBC / "soc" / "README.md",
        SOFTGBC / "soc" / "docs",
        SOFTGBC / "softgbc-core" / "src",
        SOFTGBC / "interrupt" / "README.md",
    ]
    rust_roots = [
        SOFTGBC / "softgbc-host" / "src",
        SOFTGBC / "softgbc-host" / "Cargo.toml",
        SOFTGBC / "dmg-ppu" / "src",
        SOFTGBC / "softgbc-utils" / "src",
        SOFTGBC / "peripherals" / "src",
        SOFTGBC / "soc" / "src",
        SOFTGBC / "softgbc-core" / "src",
        SOFTGBC / "cartridge" / "src",
    ]

    time.sleep(1.5)  # ignore churn from the initial serve.sh build
    docs_mt = _collect_mtimes(docs_roots)
    rust_mt = _collect_mtimes(rust_roots)
    print("watch: docs + softgbc-lab sources (poll 0.8s, debounce 1.2s)", flush=True)

    while True:
        time.sleep(0.8)
        new_docs = _collect_mtimes(docs_roots)
        new_rust = _collect_mtimes(rust_roots)
        docs_changed = new_docs != docs_mt
        rust_changed = new_rust != rust_mt
        if not docs_changed and not rust_changed:
            continue

        # Debounce: wait until mtimes settle.
        time.sleep(1.2)
        new_docs = _collect_mtimes(docs_roots)
        new_rust = _collect_mtimes(rust_roots)
        docs_changed = new_docs != docs_mt
        rust_changed = new_rust != rust_mt
        if not docs_changed and not rust_changed:
            continue

        with BUILD_LOCK:
            BUILD_STATE["busy"] = True
        try:
            if rust_changed:
                print("watch: rebuilding softgbc-lab + softgbc-play…", flush=True)
                r = subprocess.run(
                    [
                        "cargo",
                        "build",
                        "-q",
                        "-p",
                        "softgbc-host",
                        "--bin",
                        "softgbc-lab",
                        "--manifest-path",
                        str(SOFTGBC / "Cargo.toml"),
                    ],
                    cwd=str(SOFTGBC),
                    check=False,
                )
                r2 = subprocess.run(
                    [
                        "cargo",
                        "build",
                        "-q",
                        "-p",
                        "softgbc-host",
                        "--bin",
                        "softgbc-play",
                        "--release",
                        "--manifest-path",
                        str(SOFTGBC / "Cargo.toml"),
                    ],
                    cwd=str(SOFTGBC),
                    check=False,
                )
                if r2.returncode != 0:
                    r = r2
                if r.returncode != 0:
                    print("watch: cargo build failed", flush=True)
                else:
                    LabHandler.lab_bin = lab_bin.resolve()
                    get_play_mgr().shutdown_all()
                    print("watch: softgbc-lab + softgbc-play ok", flush=True)

            if docs_changed:
                print("watch: rebuilding docs site…", flush=True)
                r = subprocess.run(
                    [sys.executable, str(DOCS / "build_site.py")],
                    cwd=str(DOCS),
                    check=False,
                )
                if r.returncode != 0:
                    print("watch: build_site.py failed", flush=True)
                else:
                    print("watch: site rebuilt", flush=True)

            # Re-snapshot after rebuild so we don't re-trigger on our own writes.
            docs_mt = _collect_mtimes(docs_roots)
            rust_mt = _collect_mtimes(rust_roots)

            bid = bump_build(
                "rust"
                if rust_changed and not docs_changed
                else "docs"
                if docs_changed and not rust_changed
                else "docs+rust"
            )
            print(f"watch: build id → {bid} (browser will refresh)", flush=True)
        finally:
            with BUILD_LOCK:
                BUILD_STATE["busy"] = False


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--port", type=int, default=8000)
    ap.add_argument(
        "--bind",
        default="0.0.0.0",
        help="listen address (0.0.0.0 so Mac/LAN can reach the host, not only localhost)",
    )
    ap.add_argument("--root", type=Path, default=DEFAULT_SITE)
    ap.add_argument("--lab", type=Path, default=DEFAULT_LAB)
    ap.add_argument("--play", type=Path, default=DEFAULT_PLAY)
    ap.add_argument(
        "--watch",
        action="store_true",
        help="rebuild site/lab on changes and bump /api/dev/build for live reload",
    )
    ap.add_argument("--no-watch", action="store_true", help="disable watch (default is on via serve.sh)")
    args = ap.parse_args()

    LabHandler.site_root = args.root.resolve()
    LabHandler.lab_bin = args.lab.resolve()
    LabHandler.play_bin = args.play.resolve()
    global PLAY_MGR
    PLAY_MGR = PlaySessionManager(LabHandler.play_bin)
    if not LabHandler.site_root.is_dir():
        print(f"site root missing: {LabHandler.site_root}", file=sys.stderr)
        sys.exit(1)

    watch = args.watch and not args.no_watch
    if watch:
        t = threading.Thread(
            target=_watch_loop, args=(LabHandler.lab_bin,), daemon=True
        )
        t.start()

    global SERVER_PORT, SERVER_BIND
    SERVER_PORT = int(args.port)
    SERVER_BIND = args.bind
    httpd = LimitedThreadingHTTPServer((args.bind, args.port), LabHandler)
    host = "localhost" if args.bind in ("127.0.0.1", "0.0.0.0", "::1") else args.bind
    print(f"Docs + visual lab at http://{host}:{args.port}/")
    print(f"  Lab nav:  http://{host}:{args.port}/softgbc/environment.html")
    print(f"            http://{host}:{args.port}/softgbc/visual-tests.html")
    # LAN hint: Mac localhost ≠ this Linux box unless Cursor forwards the port.
    for lan in _lan_ips()[:2]:
        print(f"  From Mac/LAN: http://{lan}:{args.port}/softgbc/environment.html")
    print(f"  (If Cursor Remote: use the Ports panel to forward {args.port}, then Mac localhost works)")
    print(f"  site={LabHandler.site_root}")
    print(f"  lab={LabHandler.lab_bin} ({'ok' if LabHandler.lab_bin.is_file() else 'MISSING'})")
    print(f"  play={LabHandler.play_bin} ({'ok' if LabHandler.play_bin.is_file() else 'MISSING'})")
    print(f"  roms={ROMS_DIR} ({len(list_roms())} files)")
    print(f"  watch={'on' if watch else 'off'}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nbye")
    finally:
        get_play_mgr().shutdown_all()


if __name__ == "__main__":
    main()
