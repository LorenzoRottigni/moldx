#!/usr/bin/env bash
set -euo pipefail
# Show Docker container logs for a module.
# moldx docker logs <module-path> [container-name]
MODULE_PATH="${1:-.}"
CONTAINER_NAME="${2:-}"
shift 2>/dev/null || true

if [ -z "$CONTAINER_NAME" ]; then
  IMAGE_NAME=$(basename "$MODULE_PATH")
  CONTAINER_NAME="moldx-${IMAGE_NAME}"
fi

echo "[moldx] docker/logs: fetching logs for '$CONTAINER_NAME'"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  if ! docker ps -a --format '{{.Names}}' | grep -qx "$CONTAINER_NAME"; then
    echo "[moldx] docker/logs: container '$CONTAINER_NAME' not found — start it first to see real logs"
  else
    docker logs "$@" "$CONTAINER_NAME"
  fi
else
  echo "[moldx] docker/logs: 'docker' unavailable (CLI or daemon missing) — start the container to see real logs"
fi