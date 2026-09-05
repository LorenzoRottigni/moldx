#!/usr/bin/env bash
set -euo pipefail
# Test a node module.
# moldx node test <module-path> [-- <npm args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/package.json" ]; then
  echo "[moldx] node/test: no package.json in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] node/test: testing $MODULE_PATH"

if command -v npm >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && npm test "$@")
else
  echo "[moldx] node/test: 'npm' not found on PATH — install Node.js to exercise this command"
fi