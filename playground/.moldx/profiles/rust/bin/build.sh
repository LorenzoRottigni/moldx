#!/usr/bin/env bash
set -euo pipefail
# Build a Rust module.
# moldx rust build <module-path> [-- <cargo args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/Cargo.toml" ]; then
  echo "[moldx] rust/build: no Cargo.toml in $MODULE_PATH" >&2
  exit 1
fi

echo "[moldx] rust/build: building $MODULE_PATH"

if command -v cargo >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && cargo build "$@")
else
  echo "[moldx] rust/build: 'cargo' not found on PATH — install Rust to exercise this command"
fi