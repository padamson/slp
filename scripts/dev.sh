#!/usr/bin/env bash
#
# Launch both hot-reload dev servers at once:
#   - the slp-app planner   (trunk serve)     → default :8080
#   - the theoria gallery   (theoria serve)   → default :8081
#
# Ports are auto-selected: each server takes the first free port at or above
# its default, so this doesn't collide with other things you're running (the
# common case when several projects are up). Pass a preferred starting port for
# the planner as the first argument; the gallery is placed just above it.
#
# Without the theoria CLI on PATH, only the planner starts.
#
# Usage:
#   ./scripts/dev.sh            # planner from 8080, gallery just above
#   ./scripts/dev.sh 9000       # planner from 9000, gallery just above
#
# Ctrl-C stops both.
#
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

HAVE_LSOF=false
if command -v lsof >/dev/null 2>&1; then
  HAVE_LSOF=true
else
  echo "warning: lsof not found — can't check for busy ports; using defaults as-is." >&2
fi

# free_port START — echo the first TCP port >= START that nothing is LISTENing on.
free_port() {
  local port="$1"
  if "$HAVE_LSOF"; then
    while lsof -iTCP:"$port" -sTCP:LISTEN -t >/dev/null 2>&1; do
      port=$((port + 1))
    done
  fi
  echo "$port"
}

APP_PORT="$(free_port "${1:-8080}")"
echo "slp-app  (planner): http://localhost:${APP_PORT}"

if ! command -v theoria >/dev/null 2>&1; then
  echo "warning: theoria not found on PATH, so the gallery won't start. Install it with:" >&2
  echo "  cargo install --locked --git https://github.com/padamson/theoria theoria-cli" >&2
  cd crates/slp-app
  exec trunk serve --port "$APP_PORT"
fi

GALLERY_PORT="$(free_port "$((APP_PORT + 1))")"
echo "theoria  (gallery): http://localhost:${GALLERY_PORT}"
echo "Ctrl-C stops both."
echo

# Tear both servers (and their child build watchers) down together on any exit.
trap 'kill 0' EXIT

( cd crates/slp-app && exec trunk serve --port "$APP_PORT" ) &
theoria serve --port "$GALLERY_PORT" &
wait
