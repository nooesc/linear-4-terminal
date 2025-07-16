# Quick Start: Homebrew Distribution

## Immediate Steps

### 1. Create Your First Release

```bash
# Build the release
./scripts/build-release.sh

# Create and push tag
git tag -a v1.0.0 -m "Initial release: Linear CLI for terminal"
git push origin v1.0.0

# Create GitHub release with binaries
gh release create v1.0.0 \
  dist/*.tar.gz \
  --title "Linear CLI v1.0.0" \
  --notes "Initial release of Linear CLI - A terminal interface for Linear project management"
```

### 2. Create Homebrew Tap Repository

1. Go to GitHub and create a new repository named `homebrew-tap`
2. Clone it locally:
```bash
git clone https://github.com/colerafiz/homebrew-tap
cd homebrew-tap
mkdir Formula
```

### 3. Create Formula File

Create `Formula/linear-cli.rb`:
```ruby
class LinearCli < Formula
  desc "Command-line interface for Linear project management"
  homepage "https://github.com/colerafiz/linear-4-terminal"
  version "1.0.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.0.0/linear-cli-v1.0.0-macos-aarch64.tar.gz"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.0.0/linear-cli-v1.0.0-macos-x86_64.tar.gz"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  elsif OS.linux?
    url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.0.0/linear-cli-v1.0.0-linux-x86_64.tar.gz"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  end

  def install
    bin.install "linear"
  end

  test do
    assert_match "linear 1.0.0", shell_output("#{bin}/linear --version")
  end
end
```

Get the SHA256 values from `dist/checksums.txt` after building.

### 4. Test and Push

```bash
# Commit and push formula
git add Formula/linear-cli.rb
git commit -m "Add linear-cli formula v1.0.0"
git push

# Test installation
brew tap colerafiz/tap
brew install linear-cli

# Verify it works
linear --version
```

### 5. Update README

Add to your README.md:
```markdown
## Installation

### macOS/Linux via Homebrew
```bash
brew tap colerafiz/tap
brew install linear-cli
```

### Direct Download
```bash
curl -sSL https://raw.githubusercontent.com/colerafiz/linear-4-terminal/main/install.sh | bash
```

### Build from Source
```bash
cargo install --git https://github.com/colerafiz/linear-4-terminal
```
```

## That's it! 🎉

Your users can now install with:
```bash
brew tap colerafiz/tap
brew install linear-cli
```