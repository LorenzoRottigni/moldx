#!/usr/bin/env bash
set -euo pipefail
MODULE_PATH="${1:-.}"
echo "[moldx] docker/logs → $MODULE_PATH"
echo "  2026-08-04T10:00:00Z INFO  Container started"
echo "  2026-08-04T10:00:01Z INFO  Listening on :8080"
echo "  2026-08-04T10:00:05Z INFO  Health check OK"
