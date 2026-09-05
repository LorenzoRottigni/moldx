#!/usr/bin/env bash
set -euo pipefail
# Build a Docker image from a module.
# moldx docker build <module-path> [-- <additional build args>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/Dockerfile" ]; then
  echo "[moldx] docker/build: no Dockerfile in $MODULE_PATH" >&2
  exit 1
fi

IMAGE_NAME=$(basename "$MODULE_PATH")
echo "[moldx] docker/build: building '$IMAGE_NAME' in $MODULE_PATH"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker build -t "moldx-playground/$IMAGE_NAME:latest" "$@" "$MODULE_PATH"
  echo "[moldx] docker/build: tagged moldx-playground/$IMAGE_NAME:latest"
else
  echo "[moldx] docker/build: 'docker' unavailable (CLI or daemon missing) — install/start Docker to exercise this command"
fi