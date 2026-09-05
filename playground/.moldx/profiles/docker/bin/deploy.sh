#!/usr/bin/env bash
set -euo pipefail
# Deploy a Docker image to a local registry.
# moldx docker deploy <module-path> [tag]
MODULE_PATH="${1:-.}"
TAG="${2:-latest}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/Dockerfile" ]; then
  echo "[moldx] docker/deploy: no Dockerfile in $MODULE_PATH" >&2
  exit 1
fi

IMAGE_NAME=$(basename "$MODULE_PATH")
echo "[moldx] docker/deploy: deploying moldx-playground/$IMAGE_NAME:$TAG to registry.local/moldx"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker tag "moldx-playground/$IMAGE_NAME:latest" "registry.local/moldx/$IMAGE_NAME:$TAG"
  docker push "registry.local/moldx/$IMAGE_NAME:$TAG"
else
  echo "[moldx] docker/deploy: 'docker' unavailable (CLI or daemon missing) — install/start Docker to exercise this command"
fi