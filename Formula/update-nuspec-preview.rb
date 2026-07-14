class UpdateNuspecPreview < Formula
  desc "Preview build: sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  version "2.0.147-preview"
  url "https://github.com/denis-peshkov/update-nuspec-action/archive/e490fb6f75f68b03433ef6566184701b7c1b86c4.tar.gz"
  sha256 "6182665bf2b70650dcd70cd9897e237348a6c0ad150585b49fef0b6550b83485"
  license "MIT"

  depends_on "rust" => :build

  def install
    cd "update-nuspec" do
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/update-nuspec --version")
  end
end
