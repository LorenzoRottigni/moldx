#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] rust/build → $MODULE_PATH"
echo "  Running cargo build… (dry-run)"
echo "  Done."
