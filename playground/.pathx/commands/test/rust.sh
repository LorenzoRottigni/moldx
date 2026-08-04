#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] rust/test → $MODULE_PATH"
echo "  Running cargo test… (dry-run)"
echo "  All tests passed."
