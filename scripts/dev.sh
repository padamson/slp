#!/usr/bin/env bash
#
# Launch the slp-app planner's hot-reload dev server (trunk serve), default
# :8080.
#
# The port is auto-selected: the server takes the first free port at or above
# its default, so this doesn't collide with other things you're running (the
# common case when several projects are up). Pass a preferred starting port as
# the first argument.
#
# Usage:
#   ./scripts/dev.sh            # planner from 8080
#   ./scripts/dev.sh 9000       # planner from 9000
#
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Take the first TCP port at or above the start that nothing is LISTENing on.
APP_PORT="${1:-8080}"
if command -v lsof >/dev/null 2>&1; then
  while lsof -iTCP:"$APP_PORT" -sTCP:LISTEN -t >/dev/null 2>&1; do
    APP_PORT=$((APP_PORT + 1))
  done
else
  echo "warning: lsof not found — can't check for busy ports; using ${APP_PORT} as-is." >&2
fi

echo "slp-app  (planner): http://localhost:${APP_PORT}"
echo

cd crates/slp-app
exec trunk serve --port "$APP_PORT"
