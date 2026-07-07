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

CHOCO_DIR="${REPO_ROOT}/packaging/chocolatey/update-nuspec"
NUSPEC="${CHOCO_DIR}/update-nuspec.nuspec"
INSTALL_PS1="${CHOCO_DIR}/tools/chocolateyinstall.ps1"

for f in "${NUSPEC}" "${INSTALL_PS1}"; do
  if [[ ! -f "${f}" ]]; then
    echo "Template not found: ${f}" >&2
    exit 1
  fi
done

WINDOWS_URL="https://github.com/denis-peshkov/update-nuspec-action/releases/download/v${VERSION}/update-nuspec-${VERSION}-x86_64-pc-windows-msvc.zip"
WINDOWS_SHA="$(checksum_for "update-nuspec-${VERSION}-x86_64-pc-windows-msvc.zip")"

# Only the version and the release-specific url/checksum change between releases;
# everything else stays as committed in the template files.
perl -pi -e "s|<version>.*</version>|<version>${VERSION}</version>|" "${NUSPEC}"
perl -pi -e "s|(url64bit\s*=\s*').*(')|\${1}${WINDOWS_URL}\${2}|" "${INSTALL_PS1}"
perl -pi -e "s|(checksum64\s*=\s*').*(')|\${1}${WINDOWS_SHA}\${2}|" "${INSTALL_PS1}"

echo "Updated chocolatey package metadata for v${VERSION}"
