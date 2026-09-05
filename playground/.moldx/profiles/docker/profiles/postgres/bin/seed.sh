#!/usr/bin/env bash
set -euo pipefail
# Seed the PostgreSQL database from a module.
# moldx docker postgres seed <module-path>
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

if [ ! -f "$MODULE_PATH/init.sql" ]; then
  echo "[moldx] docker/postgres/seed: no init.sql in $MODULE_PATH" >&2
  exit 1
fi

IMAGE_NAME=$(basename "$MODULE_PATH")
CONTAINER_NAME="moldx-${IMAGE_NAME}"

echo "[moldx] docker/postgres/seed: seeding '${CONTAINER_NAME}' from ${MODULE_PATH}/init.sql"

if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker exec -i "$CONTAINER_NAME" psql -U playground -d playground < "$MODULE_PATH/init.sql"
else
  echo "[moldx] docker/postgres/seed: 'docker' unavailable (CLI or daemon missing) — start the database first to seed it"
fi