#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <source-tarball-sha256> <repo-root>" >&2
  exit 1
}

VERSION="${1:-}"
SOURCE_SHA256="${2:-}"
REPO_ROOT="${3:-}"

if [[ -z "${VERSION}" || -z "${SOURCE_SHA256}" || -z "${REPO_ROOT}" ]]; then
  usage
fi

FORMULA_DIR="${REPO_ROOT}/packaging/homebrew-core"
mkdir -p "${FORMULA_DIR}"

TAG="v${VERSION}"
URL="https://github.com/denis-peshkov/update-nuspec-action/archive/refs/tags/${TAG}.tar.gz"

cat > "${FORMULA_DIR}/update-nuspec.rb" <<EOF
class UpdateNuspec < Formula
  desc "Sync NuGet dependencies in nuspec files from csproj PackageReference versions"
  homepage "https://github.com/denis-peshkov/update-nuspec-action"
  url "${URL}"
  sha256 "${SOURCE_SHA256}"
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
EOF

echo "Updated homebrew-core formula draft for v${VERSION}"
