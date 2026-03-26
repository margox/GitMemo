#!/bin/bash
set -e

echo "Installing GitMemo..."

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Darwin-arm64)  BINARY="gitmemo-macos-aarch64" ;;
    Darwin-x86_64) BINARY="gitmemo-macos-x86_64" ;;
    Linux-x86_64)  BINARY="gitmemo-linux-x86_64" ;;
    Linux-aarch64) BINARY="gitmemo-linux-aarch64" ;;
    *)
        echo "Unsupported platform: ${OS}-${ARCH}"
        echo "Please build from source: https://github.com/sahadev/GitMemo#development"
        exit 1
        ;;
esac

URL="https://github.com/sahadev/GitMemo/releases/latest/download/${BINARY}"

# Download
TMPFILE=$(mktemp)
echo "Downloading ${BINARY}..."
curl -fsSL "${URL}" -o "${TMPFILE}"
chmod +x "${TMPFILE}"

# Install
INSTALL_DIR="/usr/local/bin"
if [ -w "${INSTALL_DIR}" ]; then
    mv "${TMPFILE}" "${INSTALL_DIR}/gitmemo"
else
    echo "Need sudo to install to ${INSTALL_DIR}"
    sudo mv "${TMPFILE}" "${INSTALL_DIR}/gitmemo"
fi

echo ""
echo "GitMemo installed successfully!"
echo ""
echo "Get started:"
echo "  gitmemo init"
echo ""
