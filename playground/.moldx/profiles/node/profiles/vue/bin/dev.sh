#!/usr/bin/env bash
set -euo pipefail
# Start the Vite dev server for a Vue app.
# moldx node vue dev <module-path>
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/vue/dev: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/vue/dev: starting Vite dev server in $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npm run dev "$@")
else
  echo "[moldx] node/vue/dev: 'npm' not found on PATH — install Node.js to exercise this command"
fi