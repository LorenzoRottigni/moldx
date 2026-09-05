#!/usr/bin/env bash
set -euo pipefail
# Root-level profile-agnostic command: moldx info
# Prints repository and environment information.
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "repo: $(git remote get-url origin 2>/dev/null || echo 'no remote')"
  echo "branch: $(git branch --show-current 2>/dev/null || echo 'detached')"
  echo "commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'none')"
  echo "root: $(git rev-parse --show-toplevel 2>/dev/null)"
else
  echo "repo: not a git repository"
fi
echo "shell: $BASH_VERSION"
echo "platform: $(uname -s)/$(uname -m)"
echo "cwd: $(pwd)"