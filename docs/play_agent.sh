#!/usr/bin/env bash
# SoftGBC play agent — single entry (Darwin optional, then continuous play + GDB).
# Restarts on crash so the bot stays up.
#
#   ./play_agent.sh
#   ./play_agent.sh --darwin --bots 4 --generations 3
set -uo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BASE="${SOFTGBC_PLAY_BASE:-http://127.0.0.1:8000}"
SESSION="${SOFTGBC_PLAY_SESSION:-softgbcBot1}"
SECONDS_RUN="${SOFTGBC_PLAY_SECONDS:-0}"
while true; do
  python3 "$ROOT/play_agent.py" \
    --base "$BASE" \
    --session "$SESSION" \
    --seconds "$SECONDS_RUN" \
    "$@" || true
  echo "$(date +%H:%M:%S) play_agent exited — restarting in 2s" >&2
  sleep 2
done
