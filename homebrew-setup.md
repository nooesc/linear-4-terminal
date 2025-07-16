# Homebrew Distribution Setup for Linear CLI

This guide explains how to distribute the Linear CLI tool via Homebrew, making it easily installable for users with `brew install linear-cli`.

## Prerequisites

1. **Binary releases** on GitHub with proper versioning
2. **Stable release** (not pre-release)
3. **Cross-platform binaries** (macOS Intel + Apple Silicon, optionally Linux)

## Step 1: Prepare Your Release

### 1.1 Update Version in Cargo.toml
```toml
[package]
name = "linear-cli"
version = "1.1.0"  # Semantic versioning
```

### 1.2 Create Release Binaries

Create a build script (`scripts/build-release.sh`):
```bash
#!/bin/bash
set -e

VERSION=$(grep version Cargo.toml | head -1 | cut -d'"' -f2)
PROJECT_NAME="linear-cli"

# Build for macOS Intel
echo "Building for macOS Intel..."
cargo build --release --target x86_64-apple-darwin
mkdir -p dist
cp target/x86_64-apple-darwin/release/linear dist/linear-macos-x86_64
tar -czf dist/linear-cli-v${VERSION}-macos-x86_64.tar.gz -C dist linear-macos-x86_64

# Build for macOS Apple Silicon
echo "Building for macOS Apple Silicon..."
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/linear dist/linear-macos-aarch64
tar -czf dist/linear-cli-v${VERSION}-macos-aarch64.tar.gz -C dist linear-macos-aarch64

# Calculate checksums
cd dist
shasum -a 256 *.tar.gz > checksums.txt
cd ..

echo "Release artifacts created in dist/"
```

### 1.3 Create GitHub Release

1. Create a git tag:
```bash
git tag -a v1.1.0 -m "Release version 1.1.0"
git push origin v1.1.0
```

2. Create release on GitHub:
```bash
gh release create v1.1.0 \
  --title "Linear CLI v1.1.0" \
  --notes "Release notes here..." \
  dist/*.tar.gz \
  dist/checksums.txt
```

## Step 2: Create Homebrew Formula

### Option A: Homebrew Tap (Recommended for Getting Started)

1. **Create a tap repository** on GitHub:
   - Name: `homebrew-tap` or `homebrew-linear-cli`
   - Public repository

2. **Create formula file** `Formula/linear-cli.rb`:
```ruby
class LinearCli < Formula
  desc "Command-line interface for Linear project management"
  homepage "https://github.com/colerafiz/linear-4-terminal"
  version "1.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.1.0/linear-cli-v1.1.0-macos-x86_64.tar.gz"
      sha256 "YOUR_SHA256_HERE"
    elsif Hardware::CPU.arm?
      url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.1.0/linear-cli-v1.1.0-macos-aarch64.tar.gz"
      sha256 "YOUR_SHA256_HERE"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.1.0/linear-cli-v1.1.0-linux-x86_64.tar.gz"
      sha256 "YOUR_SHA256_HERE"
    elsif Hardware::CPU.arm?
      url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.1.0/linear-cli-v1.1.0-linux-aarch64.tar.gz"
      sha256 "YOUR_SHA256_HERE"
    end
  end

  def install
    bin.install "linear"
  end

  test do
    assert_match "linear #{version}", shell_output("#{bin}/linear --version")
  end
end
```

3. **Users can install via**:
```bash
brew tap colerafiz/linear-cli
brew install linear-cli
```

### Option B: Homebrew Core (For Popular Tools)

For inclusion in homebrew-core (more visibility):
1. Tool must be notable/popular
2. Follow Homebrew's contribution guidelines
3. Submit PR to homebrew-core repository

## Step 3: Automation with GitHub Actions

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: false
          prerelease: false

  build-release:
    name: Build Release
    needs: create-release
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
            suffix: macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            suffix: macos-aarch64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            suffix: linux-x86_64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Package
        run: |
          mkdir -p dist
          cp target/${{ matrix.target }}/release/linear dist/
          tar -czf linear-cli-${{ github.ref_name }}-${{ matrix.suffix }}.tar.gz -C dist linear
          shasum -a 256 linear-cli-${{ github.ref_name }}-${{ matrix.suffix }}.tar.gz > linear-cli-${{ github.ref_name }}-${{ matrix.suffix }}.tar.gz.sha256
      
      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./linear-cli-${{ github.ref_name }}-${{ matrix.suffix }}.tar.gz
          asset_name: linear-cli-${{ github.ref_name }}-${{ matrix.suffix }}.tar.gz
          asset_content_type: application/gzip

  update-homebrew:
    name: Update Homebrew Formula
    needs: build-release
    runs-on: ubuntu-latest
    steps:
      - name: Update Homebrew Formula
        run: |
          # Script to update formula with new version and checksums
          # This would update your tap repository
```

## Step 4: Alternative Distribution Methods

### 1. **Cargo Install** (Already Working)
```bash
cargo install --git https://github.com/colerafiz/linear-4-terminal
```

### 2. **Direct Download Script**
Create `install.sh`:
```bash
#!/bin/bash
REPO="colerafiz/linear-4-terminal"
LATEST=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep tag_name | cut -d '"' -f 4)

if [[ "$OSTYPE" == "darwin"* ]]; then
  if [[ $(uname -m) == "arm64" ]]; then
    ASSET="linear-cli-${LATEST}-macos-aarch64.tar.gz"
  else
    ASSET="linear-cli-${LATEST}-macos-x86_64.tar.gz"
  fi
else
  ASSET="linear-cli-${LATEST}-linux-x86_64.tar.gz"
fi

curl -L "https://github.com/$REPO/releases/download/$LATEST/$ASSET" | tar -xz
sudo mv linear /usr/local/bin/
```

Users can install with:
```bash
curl -sSL https://raw.githubusercontent.com/colerafiz/linear-4-terminal/main/install.sh | bash
```

### 3. **Other Package Managers**

**AUR (Arch Linux):**
- Create PKGBUILD file
- Submit to AUR

**Snap:**
- Create snapcraft.yaml
- Publish to Snap Store

**Scoop (Windows):**
- Create manifest JSON
- Submit to Scoop bucket

## Step 5: Documentation Updates

Update your README.md:
```markdown
## Installation

### Homebrew (macOS/Linux)
```bash
brew tap colerafiz/linear-cli
brew install linear-cli
```

### Cargo
```bash
cargo install linear-cli
```

### Direct Download
```bash
curl -sSL https://raw.githubusercontent.com/colerafiz/linear-4-terminal/main/install.sh | bash
```

### From Source
```bash
git clone https://github.com/colerafiz/linear-4-terminal
cd linear-4-terminal
cargo install --path .
```
```

## Quick Start Steps

1. **Tag your current version**:
```bash
git tag -a v1.0.0 -m "Initial release"
git push origin v1.0.0
```

2. **Create GitHub release with binaries**:
```bash
cargo build --release
tar -czf linear-cli-v1.0.0-macos-$(uname -m).tar.gz -C target/release linear
gh release create v1.0.0 linear-cli-v1.0.0-*.tar.gz
```

3. **Create tap repository** `homebrew-linear-cli` on GitHub

4. **Add formula** and test locally:
```bash
brew tap colerafiz/linear-cli
brew install linear-cli
```

## Best Practices

1. **Semantic Versioning**: Use proper version numbers (MAJOR.MINOR.PATCH)
2. **Release Notes**: Include changelog in each release
3. **Testing**: Test formula locally before pushing
4. **Cross-platform**: Support both Intel and Apple Silicon Macs
5. **Signatures**: Consider signing releases with GPG
6. **Analytics**: Homebrew provides download analytics

## Maintenance

- Update formula for each new release
- Monitor GitHub issues for installation problems
- Consider automating formula updates with GitHub Actions
- Keep dependencies minimal for easier distribution

Once set up, users can install your Linear CLI with a simple `brew install` command!