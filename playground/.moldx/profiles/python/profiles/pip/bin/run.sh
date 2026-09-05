#!/usr/bin/env bash
set -euo pipefail
# Run the main Python script.
# moldx python pip run <module-path> [-- <script args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/requirements.txt" ]; then
  echo "[moldx] python/pip/run: no requirements.txt in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/pip/run: running main script in $MODULE_PATH"

if command -v python3 >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && python3 main.py "$@")
else
  echo "[moldx] python/pip/run: 'python3' not found on PATH — install Python to exercise this command"
fi