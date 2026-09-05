#!/usr/bin/env bash
set -euo pipefail
# Start a node module's server.
# moldx node start <module-path> [-- <npm args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/start: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/start: starting $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npm start "$@")
else
  echo "[moldx] node/start: 'npm' not found on PATH — install Node.js to exercise this command"
fi