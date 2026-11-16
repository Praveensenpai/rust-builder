#!/usr/bin/env bash
# Install rbuild (rust-builder) on Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/Praveensenpai/rust-builder/main/install.sh | bash
set -euo pipefail

REPO="Praveensenpai/rust-builder"
BINARY="rbuild"
INSTALL_DIR="${HOME}/.local/bin"

echo "Installing rbuild..."

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ]; then
  echo "Error: only Linux is supported by this installer."
  echo "You can build from source instead: cargo install --git https://github.com/${REPO}.git"
  exit 1
fi

case "$ARCH" in
  x86_64|amd64) TARGET_ARCH="x86_64" ;;
  *)
    echo "Error: only x86_64 is supported right now."
    echo "You can build from source instead: cargo install --git https://github.com/${REPO}.git"
    exit 1
    ;;
esac

TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)"
if [ -z "$TAG" ]; then
  echo "No release found."
  if command -v cargo >/dev/null 2>&1; then
    echo "Trying to install from source with cargo..."
    cargo install --git "https://github.com/${REPO}.git"
    echo "Installed rbuild with cargo."
    exit 0
  fi
  echo "Please install Rust (cargo) or wait for a release."
  exit 1
fi

ASSET="${BINARY}-${TARGET_ARCH}-linux.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"
TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT INT TERM

echo "Downloading ${BINARY} ${TAG}..."
curl -fsSL "${BASE_URL}/${ASSET}" -o "${TMP_DIR}/${ASSET}"

# Verify checksum if provided
if curl -fsSL "${BASE_URL}/${ASSET}.sha256" -o "${TMP_DIR}/${ASSET}.sha256" 2>/dev/null; then
  echo "Checking download..."
  EXPECTED="$(awk '{print $1}' "${TMP_DIR}/${ASSET}.sha256" | head -n1)"
  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
  else
    ACTUAL=""
  fi
  if [ -n "$EXPECTED" ] && [ -n "$ACTUAL" ] && [ "$EXPECTED" != "$ACTUAL" ]; then
    echo "Error: download check failed. Please try again."
    exit 1
  fi
fi

tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"
mkdir -p "$INSTALL_DIR"
install -m 755 "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"

echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

if ! echo ":$PATH:" | grep -q ":${INSTALL_DIR}:"; then
  echo "Note: ${INSTALL_DIR} is not in your PATH."
  echo "Add this line to your ~/.bashrc or ~/.zshrc, then restart your terminal:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo "Done. Run: rbuild --help"
