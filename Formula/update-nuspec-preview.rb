class UpdateNuspecPreview < Formula
  desc "Preview build: sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  version "2.0.123-preview"
  url "https://github.com/denis-peshkov/update-nuspec-action/archive/4c40415635cbdd342db8a6d290fe23c1dd8953f3.tar.gz"
  sha256 "09429e850697ddefc3015fc199e7f2dc2cbb81dd43947a9f0b504a6970bfa3ce"
  license "MIT"

  depends_on "rust" => :build

  def install
    cd "update-nuspec" do
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    assert_match "update-nuspec", shell_output("#{bin}/update-nuspec --version")
  end
end
