#!/usr/bin/env bash
# Detect strategies by looking for well-known marker files in $1
TARGET="$1"
[ -f "$TARGET/Dockerfile" ]   && echo "docker"
[ -f "$TARGET/package.json" ] && echo "node"
[ -f "$TARGET/Cargo.toml" ]   && echo "rust"
exit 0
