#!/usr/bin/env bash
set -euo pipefail
# Run a Python script with uv.
# moldx python uv run <module-path> [-- <script args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/pyproject.toml" ]; then
  echo "[moldx] python/uv/run: no pyproject.toml in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/uv/run: running main script in $MODULE_PATH"

if command -v uv >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && uv run python main.py "$@")
else
  echo "[moldx] python/uv/run: 'uv' not found on PATH — install uv to exercise this command"
fi