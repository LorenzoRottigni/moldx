#!/usr/bin/env bash
set -euo pipefail
# Profile-agnostic diff command (matches any module via the root profile).
# moldx diff <module-path> [-- <git diff options>]
MODULE_PATH="${1:-.}"
shift 2>/dev/null || true

echo "[moldx] agnostic/diff -> $MODULE_PATH"

if command -v git >/dev/null 2>&1 && [ -d "$MODULE_PATH/.git" ]; then
  (cd "$MODULE_PATH" && git diff --stat HEAD~1 HEAD || git diff --stat)
elif command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
  (cd "$MODULE_PATH" && git diff --stat HEAD~1 HEAD || git diff --stat)
else
  echo "[moldx] agnostic/diff: '$MODULE_PATH' is not inside a git work tree — nothing to diff"
fi