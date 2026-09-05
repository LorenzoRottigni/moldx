#!/usr/bin/env bash
set -euo pipefail
# Show the status of a PostgreSQL container.
# moldx docker postgres status <module-path>
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

IMAGE_NAME=$(basename "$MODULE_PATH")
CONTAINER_NAME="moldx-${IMAGE_NAME}"

echo "[moldx] docker/postgres/status: checking '${CONTAINER_NAME}'"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker ps --filter "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
else
  echo "[moldx] docker/postgres/status: 'docker' unavailable (CLI or daemon missing) — install/start Docker to exercise this command"
fi