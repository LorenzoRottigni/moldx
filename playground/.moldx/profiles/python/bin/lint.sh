#!/usr/bin/env bash
set -euo pipefail
# Lint a Python module.
# moldx python lint <module-path> [-- <lint args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -d "$MODULE_PATH" ]; then
  echo "[moldx] python/lint: module path does not exist: $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] python/lint: checking $MODULE_PATH"

if command -v ruff >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && ruff check "$@")
elif command -v flake8 >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && flake8 "$@")
else
  echo "[moldx] python/lint: 'ruff'/'flake8' not found on PATH — install one to exercise this command"
fi