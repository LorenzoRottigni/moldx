#!/usr/bin/env bash
set -euo pipefail
# Start the Nuxt dev server.
# moldx node nuxt dev <module-path>
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/nuxt/dev: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/nuxt/dev: starting Nuxt dev server in $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npx nuxt dev "$@")
else
  echo "[moldx] node/nuxt/dev: 'npm' not found on PATH — install Node.js to exercise this command"
fi