class Setups < Formula
  desc "Pre-market trade setup scanner: gaps, relative volume, catalyst scoring"
  homepage "https://github.com/holynakamoto/setups"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/holynakamoto/setups/releases/download/v#{version}/setups-aarch64-apple-darwin.tar.gz"
      sha256 "4434a41350ca4700a9c9991d87ad6d708456bf1ed2d8c5bed0ca8adcc8636fad"
    end

    on_intel do
      url "https://github.com/holynakamoto/setups/releases/download/v#{version}/setups-x86_64-apple-darwin.tar.gz"
      sha256 "2ee81ce58b2fe9146c985cb3d6ac7521910d9cef47dea1981be2994e97b965db"
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
