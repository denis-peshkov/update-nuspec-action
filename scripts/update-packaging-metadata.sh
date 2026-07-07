#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <checksums-file> <repo-root>" >&2
  exit 1
}

VERSION="${1:-}"
CHECKSUMS_FILE="${2:-}"
REPO_ROOT="${3:-}"

if [[ -z "${VERSION}" || -z "${CHECKSUMS_FILE}" || -z "${REPO_ROOT}" ]]; then
  usage
fi

if [[ ! -f "${CHECKSUMS_FILE}" ]]; then
  echo "Checksums file not found: ${CHECKSUMS_FILE}" >&2
  exit 1
fi

checksum_for() {
  local suffix="$1"
  local line
  line="$(grep " ${suffix}$" "${CHECKSUMS_FILE}" | head -n 1 || true)"
  if [[ -z "${line}" ]]; then
    echo "Checksum not found for artifact suffix: ${suffix}" >&2
    exit 1
  fi
  echo "${line%% *}"
}

RELEASE_BASE="https://github.com/denis-peshkov/update-nuspec-action/releases/download/v${VERSION}"

WINDOWS_SHA="$(checksum_for "update-nuspec-${VERSION}-x86_64-pc-windows-msvc.zip")"

CHOCO_DIR="${REPO_ROOT}/packaging/chocolatey/update-nuspec"
mkdir -p "${CHOCO_DIR}/tools"

cat > "${CHOCO_DIR}/update-nuspec.nuspec" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>update-nuspec</id>
    <version>${VERSION}</version>
    <title>update-nuspec</title>
    <authors>Denis Peshkov</authors>
    <owners>Denis Peshkov</owners>
    <projectUrl>https://github.com/denis-peshkov/update-nuspec-action</projectUrl>
    <iconUrl>https://raw.githubusercontent.com/denis-peshkov/update-nuspec-action/master/update-nuspec-icon.png</iconUrl>
    <license type="expression">MIT</license>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <summary>Sync NuGet dependencies in nuspec files from csproj PackageReference versions.</summary>
    <description>CLI to sync NuGet dependencies in *.nuspec with PackageReference versions from matching *.csproj files. Optionally updates package.json version and scoped npm dependencies.</description>
    <releaseNotes>https://github.com/denis-peshkov/update-nuspec-action/releases/tag/v${VERSION}</releaseNotes>
    <tags>nuget nuspec dotnet cli csproj</tags>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
EOF

cat > "${CHOCO_DIR}/tools/chocolateyinstall.ps1" <<EOF
\$ErrorActionPreference = 'Stop'

\$toolsDir = "\$(Split-Path -Parent \$MyInvocation.MyCommand.Definition)"
\$packageArgs = @{
  packageName    = \$env:ChocolateyPackageName
  unzipLocation  = \$toolsDir
  url64bit       = '${RELEASE_BASE}/update-nuspec-${VERSION}-x86_64-pc-windows-msvc.zip'
  checksum64     = '${WINDOWS_SHA}'
  checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
EOF

cat > "${CHOCO_DIR}/tools/chocolateyuninstall.ps1" <<'EOF'
$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
Uninstall-ChocolateyZipPackage -PackageName $env:ChocolateyPackageName -ZipFileName "$toolsDir\update-nuspec.exe"
EOF

echo "Updated chocolatey package metadata for v${VERSION}"
