#!/usr/bin/env bash
set -euo pipefail
# Test a Python module.
# moldx python test <module-path> [-- <pytest args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -d "$MODULE_PATH" ]; then
  echo "[moldx] python/test: module path does not exist: $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/test: testing $MODULE_PATH"

if command -v pytest >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && pytest "$@")
elif command -v python3 >/dev/null 2>&1 && python3 -c 'import pytest' 2>/dev/null; then
  (cd "$MODULE_PATH" && python3 -m pytest "$@")
else
  echo "[moldx] python/test: 'pytest' not found on PATH — install it to exercise this command"
fi