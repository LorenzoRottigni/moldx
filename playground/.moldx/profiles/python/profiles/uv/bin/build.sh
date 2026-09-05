#!/usr/bin/env bash
set -euo pipefail
# Build a Python package with uv.
# moldx python uv build <module-path> [-- <uv args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/pyproject.toml" ]; then
  echo "[moldx] python/uv/build: no pyproject.toml in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/uv/build: building package in $MODULE_PATH"

if command -v uv >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && uv build "$@")
else
  echo "[moldx] python/uv/build: 'uv' not found on PATH — install uv to exercise this command"
fi