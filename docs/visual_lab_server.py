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
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlparse

DOCS = Path(__file__).resolve().parent
SOFTGBC = DOCS.parent / "SoftGBC"
DEFAULT_SITE = DOCS / "site"
DEFAULT_LAB = SOFTGBC / "target" / "debug" / "softgbc-lab"
FIXTURES = SOFTGBC / "dmg-ppu" / "fixtures"

# Shared with the watcher thread.
BUILD_STATE = {"id": 1, "msg": "boot", "busy": False}
BUILD_LOCK = threading.Lock()


def bump_build(msg: str) -> int:
    with BUILD_LOCK:
        BUILD_STATE["id"] = int(BUILD_STATE["id"]) + 1
        BUILD_STATE["msg"] = msg
        return int(BUILD_STATE["id"])


class LabHandler(BaseHTTPRequestHandler):
    site_root: Path = DEFAULT_SITE
    lab_bin: Path = DEFAULT_LAB

    def log_message(self, fmt: str, *args) -> None:
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    def _send(self, code: int, body: bytes, content_type: str) -> None:
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _json(self, code: int, obj: object) -> None:
        data = json.dumps(obj).encode("utf-8")
        self._send(code, data, "application/json; charset=utf-8")

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
                print("watch: rebuilding softgbc-lab…", flush=True)
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
                if r.returncode != 0:
                    print("watch: cargo build failed", flush=True)
                else:
                    LabHandler.lab_bin = lab_bin.resolve()
                    print("watch: softgbc-lab ok", flush=True)

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
    ap.add_argument("--bind", default="127.0.0.1")
    ap.add_argument("--root", type=Path, default=DEFAULT_SITE)
    ap.add_argument("--lab", type=Path, default=DEFAULT_LAB)
    ap.add_argument(
        "--watch",
        action="store_true",
        help="rebuild site/lab on changes and bump /api/dev/build for live reload",
    )
    ap.add_argument("--no-watch", action="store_true", help="disable watch (default is on via serve.sh)")
    args = ap.parse_args()

    LabHandler.site_root = args.root.resolve()
    LabHandler.lab_bin = args.lab.resolve()
    if not LabHandler.site_root.is_dir():
        print(f"site root missing: {LabHandler.site_root}", file=sys.stderr)
        sys.exit(1)

    watch = args.watch and not args.no_watch
    if watch:
        t = threading.Thread(
            target=_watch_loop, args=(LabHandler.lab_bin,), daemon=True
        )
        t.start()

    httpd = ThreadingHTTPServer((args.bind, args.port), LabHandler)
    host = "localhost" if args.bind in ("127.0.0.1", "0.0.0.0", "::1") else args.bind
    print(f"Docs + visual lab at http://{host}:{args.port}/")
    print(f"  Lab nav:  http://{host}:{args.port}/softgbc/environment.html")
    print(f"            http://{host}:{args.port}/softgbc/visual-tests.html")
    print(f"  site={LabHandler.site_root}")
    print(f"  lab={LabHandler.lab_bin} ({'ok' if LabHandler.lab_bin.is_file() else 'MISSING'})")
    print(f"  watch={'on' if watch else 'off'}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nbye")


if __name__ == "__main__":
    main()
