#!/usr/bin/env bash
set -euo pipefail
# Install Python dependencies with uv.
# moldx python uv install <module-path> [-- <uv args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/pyproject.toml" ]; then
  echo "[moldx] python/uv/install: no pyproject.toml in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/uv/install: installing dependencies in $MODULE_PATH"

if command -v uv >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && uv sync "$@")
else
  echo "[moldx] python/uv/install: 'uv' not found on PATH — install uv to exercise this command"
fi