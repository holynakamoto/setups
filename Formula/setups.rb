class Setups < Formula
  desc "Pre-market trade setup scanner: gaps, relative volume, catalyst scoring"
  homepage "https://github.com/holynakamoto/setups"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      # Replace PLACEHOLDER_SHA256 with the sha256 of the arm64 tarball before publishing.
      url "https://github.com/holynakamoto/setups/releases/download/v#{version}/setups-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_AARCH64"
    end

    on_intel do
      # Replace PLACEHOLDER_SHA256 with the sha256 of the x86_64 tarball before publishing.
      url "https://github.com/holynakamoto/setups/releases/download/v#{version}/setups-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  def install
    bin.install "setups"
  end

  def caveats
    <<~EOS
      setups requires a Finnhub API key to fetch quotes and news.

      Set the environment variable before running:
        export FINNHUB_API_KEY=your_key_here

      A free key is available at https://finnhub.io.

      You can also create a .env file in the directory where you run setups.
      See the bundled .env.example for the expected format:
        https://github.com/holynakamoto/setups/blob/master/.env.example
    EOS
  end

  test do
    assert_match "setups", shell_output("#{bin}/setups --help", 0)
  end
end
