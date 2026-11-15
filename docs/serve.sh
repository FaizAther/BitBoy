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
echo "Building softgbc-lab…"
cargo build -q -p softgbc-host --bin softgbc-lab --manifest-path ../SoftGBC/Cargo.toml

if [[ "$BUILD_ONLY" -eq 1 ]]; then
  echo "Built docs/site/ + softgbc-lab (not serving)"
  exit 0
fi

WATCH_FLAG=(--watch)
if [[ "$WATCH" -eq 0 ]]; then
  WATCH_FLAG=(--no-watch)
fi

echo "Docs + lab:"
echo "  http://localhost:${PORT}/softgbc/environment.html"
echo "  http://localhost:${PORT}/softgbc/visual-tests.html"
exec .venv/bin/python visual_lab_server.py --port "$PORT" --root site --lab "$LAB_BIN" "${WATCH_FLAG[@]}"
