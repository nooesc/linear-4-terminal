class LinearCli < Formula
  desc "Command-line interface for Linear project management"
  homepage "https://github.com/colerafiz/linear-4-terminal"
  version "1.0.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.0.0/linear-cli-v1.0.0-macos-aarch64.tar.gz"
    sha256 "f1e2683374a6e93d506aebb67c746e75643829885dfe8233d34a6e1f454b18dd"
  elsif OS.mac? && Hardware::CPU.intel?
    # Note: You'll need to build this on an Intel Mac or use cross-compilation
    url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.0.0/linear-cli-v1.0.0-macos-x86_64.tar.gz"
    sha256 "REPLACE_WITH_INTEL_MAC_SHA256"
  elsif OS.linux?
    # Note: You'll need to build this on Linux or use cross-compilation
    url "https://github.com/colerafiz/linear-4-terminal/releases/download/v1.0.0/linear-cli-v1.0.0-linux-x86_64.tar.gz"
    sha256 "REPLACE_WITH_LINUX_SHA256"
  end

  def install
    bin.install "linear"
  end

  test do
    assert_match "linear 1.0.0", shell_output("#{bin}/linear --version")
  end
end