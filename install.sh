#!/usr/bin/env bash
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/LorenzoRottigni/moldx/main/install.sh | bash
#
# Environment variables (all optional):
#   MOLDX_REPO         GitHub "owner/repo"   default: LorenzoRottigni/moldx
#   MOLDX_VERSION      Tag to install        default: latest release
#   MOLDX_INSTALL_DIR  Directory to install  default: /usr/local/bin
#   GITHUB_TOKEN       API token             reduces rate-limit risk

set -euo pipefail

REPO="${MOLDX_REPO:-LorenzoRottigni/moldx}"
INSTALL_DIR="${MOLDX_INSTALL_DIR:-/usr/local/bin}"
BINARY="moldx"

# ── Detect platform ──────────────────────────────────────────────────────────

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)  PLATFORM="linux"  ;;
  Darwin) PLATFORM="macos"  ;;
  *)
    echo "error: unsupported OS '${OS}'" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_LABEL="x86_64"  ;;
  aarch64|arm64) ARCH_LABEL="aarch64" ;;
  *)
    echo "error: unsupported architecture '${ARCH}'" >&2
    exit 1
    ;;
esac

ASSET_NAME="${BINARY}-${PLATFORM}-${ARCH_LABEL}"

# ── Resolve release tag ──────────────────────────────────────────────────────

TAG="${MOLDX_VERSION:-}"

if [ -z "$TAG" ]; then
  AUTH_HEADER=""
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    AUTH_HEADER="-H \"Authorization: Bearer ${GITHUB_TOKEN}\""
  fi

  TAG=$(curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    ${AUTH_HEADER:+"$AUTH_HEADER"} \
    "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
fi

if [ -z "$TAG" ]; then
  echo "error: could not determine latest release tag." >&2
  echo "       Set MOLDX_VERSION=vX.Y.Z or check your GitHub API rate limit." >&2
  exit 1
fi

# ── Download ─────────────────────────────────────────────────────────────────

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET_NAME}"

TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT

echo "moldx ${TAG} (${PLATFORM}/${ARCH_LABEL})"
echo "Downloading from ${DOWNLOAD_URL} ..."

if ! curl -fsSL "$DOWNLOAD_URL" -o "$TMP"; then
  echo "error: download failed. Check that ${TAG} has a '${ASSET_NAME}' asset." >&2
  exit 1
fi

chmod +x "$TMP"

# ── Install ──────────────────────────────────────────────────────────────────

DEST="${INSTALL_DIR}/${BINARY}"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP" "$DEST"
else
  echo "Installing to ${DEST} (requires sudo)..."
  sudo mv "$TMP" "$DEST"
fi

echo ""
echo "Installed: $(command -v "$BINARY" 2>/dev/null || echo "$DEST")"
echo "Version:   $("$DEST" --version 2>/dev/null || echo "$TAG")"
echo ""
echo "Run 'moldx --help' to get started."
