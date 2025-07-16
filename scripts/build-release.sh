#!/bin/bash
set -e

# Get version from Cargo.toml
VERSION=$(grep version Cargo.toml | head -1 | cut -d'"' -f2)
PROJECT_NAME="linear-cli"

echo "Building Linear CLI v${VERSION} for release..."

# Create dist directory
mkdir -p dist
rm -f dist/*

# Detect current platform for native build
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(uname -m) == "arm64" ]]; then
        echo "Building for macOS Apple Silicon..."
        cargo build --release
        cp target/release/linear dist/linear
        tar -czf dist/${PROJECT_NAME}-v${VERSION}-macos-aarch64.tar.gz -C dist linear
    else
        echo "Building for macOS Intel..."
        cargo build --release
        cp target/release/linear dist/linear
        tar -czf dist/${PROJECT_NAME}-v${VERSION}-macos-x86_64.tar.gz -C dist linear
    fi
else
    echo "Building for Linux..."
    cargo build --release
    cp target/release/linear dist/linear
    tar -czf dist/${PROJECT_NAME}-v${VERSION}-linux-x86_64.tar.gz -C dist linear
fi

# Calculate checksums
cd dist
shasum -a 256 *.tar.gz > checksums.txt
echo ""
echo "Release artifacts created:"
ls -la
echo ""
echo "SHA256 checksums:"
cat checksums.txt
cd ..

echo ""
echo "Next steps:"
echo "1. Create a git tag: git tag -a v${VERSION} -m \"Release version ${VERSION}\""
echo "2. Push the tag: git push origin v${VERSION}"
echo "3. Create GitHub release: gh release create v${VERSION} dist/*.tar.gz --title \"Linear CLI v${VERSION}\" --notes \"Release notes here...\""