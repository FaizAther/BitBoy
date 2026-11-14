#!/usr/bin/env bash
# Build static docs from Markdown, then serve docs/site/
set -euo pipefail
cd "$(dirname "$0")"

PORT=8000
BUILD_ONLY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-only) BUILD_ONLY=1; shift ;;
    --port) PORT="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: ./serve.sh [--build-only] [--port N]"
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

if [[ "$BUILD_ONLY" -eq 1 ]]; then
  echo "Built docs/site/ (not serving)"
  exit 0
fi

echo "Serving docs/site at http://127.0.0.1:${PORT}/  (Ctrl-C to stop)"
cd site
exec python3 -m http.server "$PORT"
