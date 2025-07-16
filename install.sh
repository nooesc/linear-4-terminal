#!/bin/bash
set -e

REPO="colerafiz/linear-4-terminal"
BINARY_NAME="linear"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Installing Linear CLI..."

# Get latest release
LATEST=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep tag_name | cut -d '"' -f 4)

if [ -z "$LATEST" ]; then
    echo -e "${RED}Error: Could not fetch latest release${NC}"
    exit 1
fi

echo "Latest version: $LATEST"

# Detect OS and architecture
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
    if [[ $(uname -m) == "arm64" ]]; then
        ARCH="aarch64"
    else
        ARCH="x86_64"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
    ARCH="x86_64"
else
    echo -e "${RED}Unsupported operating system: $OSTYPE${NC}"
    exit 1
fi

ASSET="linear-cli-${LATEST}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST/$ASSET"

echo "Downloading $ASSET..."

# Create temp directory
TEMP_DIR=$(mktemp -d)
cd $TEMP_DIR

# Download and extract
if ! curl -L -o "$ASSET" "$DOWNLOAD_URL"; then
    echo -e "${RED}Error: Failed to download release${NC}"
    echo "URL: $DOWNLOAD_URL"
    exit 1
fi

tar -xzf "$ASSET"

# Install binary
echo "Installing to /usr/local/bin/..."
if [ -w "/usr/local/bin" ]; then
    mv $BINARY_NAME /usr/local/bin/
else
    echo -e "${YELLOW}Need sudo access to install to /usr/local/bin${NC}"
    sudo mv $BINARY_NAME /usr/local/bin/
fi

# Clean up
cd ..
rm -rf $TEMP_DIR

# Verify installation
if command -v $BINARY_NAME &> /dev/null; then
    echo -e "${GREEN}✓ Linear CLI installed successfully!${NC}"
    echo "Version: $($BINARY_NAME --version)"
else
    echo -e "${RED}Installation may have failed. Please check your PATH.${NC}"
    exit 1
fi