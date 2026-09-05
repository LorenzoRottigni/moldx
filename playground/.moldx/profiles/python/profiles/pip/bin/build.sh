#!/usr/bin/env bash
set -euo pipefail
# Build a Python package with pip tooling.
# moldx python pip build <module-path> [-- <build args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/requirements.txt" ]; then
  echo "[moldx] python/pip/build: no requirements.txt in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/pip/build: building package in $MODULE_PATH"

if command -v python3 >/dev/null 2>&1 && python3 -c 'import build' 2>/dev/null; then
  (cd "$MODULE_PATH" && python3 -m build "$@")
else
  echo "[moldx] python/pip/build: 'python-build' not available — pip install build to exercise this command"
fi