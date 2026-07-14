class UpdateNuspecPreview < Formula
  desc "Preview build: sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  version "2.0.144-preview"
  url "https://github.com/denis-peshkov/update-nuspec-action/archive/9c7a81965feadb5807d6807c704558e04510848e.tar.gz"
  sha256 "79b99383bdfdd1ad6f2aa71f8107b148486db708ee70a9bbeaf4f77fafb24170"
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
