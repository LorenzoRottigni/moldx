#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] node/install → $MODULE_PATH"
echo "  Installing npm packages… (dry-run)"
echo "  Done."
