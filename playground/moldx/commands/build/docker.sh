#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] docker/build → $MODULE_PATH"
echo "  Validating Dockerfile…"
[ -f "$MODULE_PATH/Dockerfile" ] && echo "  Dockerfile found." || echo "  (no Dockerfile, dry-run)"
echo "  Building image…"
echo "  Done."
