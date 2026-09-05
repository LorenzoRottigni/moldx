#!/usr/bin/env bash
set -euo pipefail

REPO="${MOLDX_REPO:-LorenzoRottigni/moldx}"
INSTALL_DIR="${MOLDX_INSTALL_DIR:-/usr/local/bin}"
BINARY="moldx"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux) PLATFORM="linux" ;;
  Darwin) PLATFORM="macos" ;;
  *)
    echo "error: unsupported OS '${OS}'. Use install.ps1 on Windows." >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_LABEL="x86_64" ;;
  aarch64|arm64) ARCH_LABEL="aarch64" ;;
  *)
    echo "error: unsupported architecture '${ARCH}'" >&2
    exit 1
    ;;
esac

TAG="${MOLDX_VERSION:-}"
if [ -z "$TAG" ]; then
  TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' \
    | head -n 1)
fi

if [ -z "$TAG" ]; then
  echo "error: could not determine the release version" >&2
  exit 1
fi

ASSET_NAME="${BINARY}-${PLATFORM}-${ARCH_LABEL}"
ARCHIVE_NAME="${ASSET_NAME}-${TAG}.tar.gz"
DOWNLOAD_BASE="https://github.com/${REPO}/releases/download/${TAG}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

printf 'Installing %s %s (%s/%s)\n' "$BINARY" "$TAG" "$PLATFORM" "$ARCH_LABEL"
curl -fsSL "${DOWNLOAD_BASE}/${ARCHIVE_NAME}" -o "${TMP_DIR}/${ARCHIVE_NAME}"
curl -fsSL "${DOWNLOAD_BASE}/SHA256SUMS" -o "${TMP_DIR}/SHA256SUMS"
(
  cd "$TMP_DIR"
  grep "  ${ARCHIVE_NAME}$" SHA256SUMS | sha256sum -c -
)
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"
chmod +x "${TMP_DIR}/${BINARY}"

DEST="${INSTALL_DIR}/${BINARY}"
if [ -w "$INSTALL_DIR" ]; then
  mv "${TMP_DIR}/${BINARY}" "$DEST"
else
  echo "Installing to ${DEST} (requires sudo)..."
  sudo mkdir -p "$INSTALL_DIR"
  sudo mv "${TMP_DIR}/${BINARY}" "$DEST"
fi

printf 'Installed %s at %s\n' "$BINARY" "$DEST"
"$DEST" --version
