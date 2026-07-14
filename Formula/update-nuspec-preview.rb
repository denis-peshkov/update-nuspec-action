class UpdateNuspecPreview < Formula
  desc "Preview build: sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  version "2.0.145-preview"
  url "https://github.com/denis-peshkov/update-nuspec-action/archive/11865b5977201c46ca0ef5f5c0c0dcceb924c5fd.tar.gz"
  sha256 "48de8df53f1550c297ce3571d1d7ffc98ee5926b40e485e602a819c1787b191b"
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
