#!/usr/bin/env bash
# Build static docs from Markdown, build softgbc-lab, then serve with visual API.
# Default: --watch rebuilds site/lab on changes; browser live_reload.js refreshes.
set -euo pipefail
cd "$(dirname "$0")"

PORT=8000
BUILD_ONLY=0
WATCH=1
while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-only) BUILD_ONLY=1; shift ;;
    --port) PORT="$2"; shift 2 ;;
    --watch) WATCH=1; shift ;;
    --no-watch) WATCH=0; shift ;;
    -h|--help)
      echo "Usage: ./serve.sh [--build-only] [--port N] [--watch|--no-watch]"
      exit 0
      ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

if [[ ! -d .venv ]]; then
  python3 -m venv .venv
fi
.venv/bin/pip -q install -r requirements.txt
.venv/bin/python build_site.py

LAB_BIN="../SoftGBC/target/debug/softgbc-lab"
# Release play binary — debug is ~30fps and feels choppy in the browser.
PLAY_BIN="../SoftGBC/target/release/softgbc-play"
echo "Building softgbc-lab (debug) + softgbc-play (release)…"
cargo build -q -p softgbc-host --bin softgbc-lab --manifest-path ../SoftGBC/Cargo.toml
cargo build -q -p softgbc-host --bin softgbc-play --release --manifest-path ../SoftGBC/Cargo.toml

if [[ "$BUILD_ONLY" -eq 1 ]]; then
  echo "Built docs/site/ + softgbc-lab + softgbc-play (not serving)"
  exit 0
fi

WATCH_FLAG=(--watch)
if [[ "$WATCH" -eq 0 ]]; then
  WATCH_FLAG=(--no-watch)
fi

LAN_IP="$(hostname -I 2>/dev/null | awk '{print $1}')"
echo "Docs + lab + play:"
echo "  http://localhost:${PORT}/softgbc/environment.html"
echo "  http://localhost:${PORT}/softgbc/visual-tests.html"
if [[ -n "${LAN_IP}" ]]; then
  echo "  From Mac/LAN (not Mac-localhost): http://${LAN_IP}:${PORT}/softgbc/environment.html"
fi
echo "  Or forward port ${PORT} in Cursor Ports, then use Mac http://localhost:${PORT}/…"
exec .venv/bin/python visual_lab_server.py --port "$PORT" --bind 0.0.0.0 --root site --lab "$LAB_BIN" --play "$PLAY_BIN" "${WATCH_FLAG[@]}"
