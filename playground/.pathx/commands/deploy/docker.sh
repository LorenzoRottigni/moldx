#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] docker/deploy → $MODULE_PATH"
echo "  Pushing image to registry…"
echo "  Updating deployment…"
echo "  Done."
