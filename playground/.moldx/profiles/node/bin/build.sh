#!/usr/bin/env bash
set -euo pipefail
# Build a node module.
# moldx node build <module-path> [-- <npm args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/build: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/build: building $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npm run build "$@")
else
  echo "[moldx] node/build: 'npm' not found on PATH — install Node.js to exercise this command"
fi