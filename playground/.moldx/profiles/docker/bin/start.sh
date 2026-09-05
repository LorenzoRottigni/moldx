#!/usr/bin/env bash
set -euo pipefail
# Start a Docker container for a module.
# moldx docker start <module-path> [container-name]
MODULE_PATH="${1:-.}"
CONTAINER_NAME="${2:-}"
shift 2>/dev/null || true

if [ -z "$CONTAINER_NAME" ]; then
  IMAGE_NAME=$(basename "$MODULE_PATH")
  CONTAINER_NAME="moldx-${IMAGE_NAME}"
fi

IMAGE_NAME=$(basename "$MODULE_PATH")
echo "[moldx] docker/start: starting '$CONTAINER_NAME' from moldx-playground/$IMAGE_NAME:latest"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker run -d --name "$CONTAINER_NAME" "moldx-playground/$IMAGE_NAME:latest"
else
  echo "[moldx] docker/start: 'docker' unavailable (CLI or daemon missing) — install/start Docker to exercise this command"
fi