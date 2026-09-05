#!/usr/bin/env bash
set -euo pipefail
# Stop a Docker container for a module.
# moldx docker stop <module-path> [container-name]
MODULE_PATH="${1:-.}"
CONTAINER_NAME="${2:-}"
shift 2>/dev/null || true

if [ -z "$CONTAINER_NAME" ]; then
  IMAGE_NAME=$(basename "$MODULE_PATH")
  CONTAINER_NAME="moldx-${IMAGE_NAME}"
fi

echo "[moldx] docker/stop: stopping '$CONTAINER_NAME'"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker rm -f "$CONTAINER_NAME" 2>/dev/null || true
else
  echo "[moldx] docker/stop: 'docker' unavailable (CLI or daemon missing) — install/start Docker to exercise this command"
fi