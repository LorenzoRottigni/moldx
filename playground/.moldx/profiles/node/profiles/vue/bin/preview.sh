#!/usr/bin/env bash
set -euo pipefail
# Preview a built Vue app.
# moldx node vue preview <module-path>
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/vue/preview: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/vue/preview: previewing built app in $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npm run preview "$@")
else
  echo "[moldx] node/vue/preview: 'npm' not found on PATH — install Node.js to exercise this command"
fi