class UpdateNuspecPreview < Formula
  desc "Preview build: sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  version "2.0.121-preview"
  url "https://github.com/denis-peshkov/update-nuspec-action/archive/45315e8e71ba5c06496baa2839dc36cac138b412.tar.gz"
  sha256 "21e0c36076a748e82720444fc1e77949633ec9679b91176df1bc9bb511f65c4b"
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
