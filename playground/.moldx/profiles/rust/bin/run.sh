#!/usr/bin/env bash
set -euo pipefail
# Run a Rust module.
# moldx rust run <module-path> [-- <cargo args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/Cargo.toml" ]; then
  echo "[moldx] rust/run: no Cargo.toml in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] rust/run: running $MODULE_PATH"

if command -v cargo >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && cargo run "$@")
else
  echo "[moldx] rust/run: 'cargo' not found on PATH — install Rust to exercise this command"
fi