#!/usr/bin/env bash
set -euo pipefail
# Install Python dependencies with pip.
# moldx python pip install <module-path> [-- <pip args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/requirements.txt" ]; then
  echo "[moldx] python/pip/install: no requirements.txt in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/pip/install: installing dependencies in $MODULE_PATH"

if command -v python3 >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && python3 -m pip install -r requirements.txt "$@")
else
  echo "[moldx] python/pip/install: 'python3' not found on PATH — install Python to exercise this command"
fi