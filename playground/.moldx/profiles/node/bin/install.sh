#!/usr/bin/env bash
set -euo pipefail
# Install npm dependencies for a node module.
# moldx node install <module-path> [-- <npm args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/install: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/install: installing dependencies in $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npm install "$@")
else
  echo "[moldx] node/install: 'npm' not found on PATH — install Node.js to exercise this command"
fi