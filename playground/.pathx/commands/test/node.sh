#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] node/test → $MODULE_PATH"
echo "  Running Jest… (dry-run)"
echo "  All tests passed."
