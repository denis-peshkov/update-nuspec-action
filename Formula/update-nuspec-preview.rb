class UpdateNuspecPreview < Formula
  desc "Preview build: sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  version "2.0.148-preview"
  url "https://github.com/denis-peshkov/update-nuspec-action/archive/647bc2f308e818304b4a7f6b720b975098eedeb4.tar.gz"
  sha256 "8ba5cce53c485dc896b20b4895512adfd6c3d3f928bb722eb898e2ba5472d419"
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
