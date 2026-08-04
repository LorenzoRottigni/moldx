#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] docker/stop → $MODULE_PATH"
echo "  Stopping container…"
echo "  Done."
