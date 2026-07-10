#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <windows-zip> <repo-root> <output-dir>" >&2
  exit 1
}

VERSION="${1:-}"
WINDOWS_ZIP="${2:-}"
REPO_ROOT="${3:-}"
OUTPUT_DIR="${4:-}"

if [[ -z "${VERSION}" || -z "${WINDOWS_ZIP}" || -z "${REPO_ROOT}" || -z "${OUTPUT_DIR}" ]]; then
  usage
fi

if [[ ! -f "${WINDOWS_ZIP}" ]]; then
  echo "Windows release archive not found: ${WINDOWS_ZIP}" >&2
  exit 1
fi

TEMPLATE_DIR="${REPO_ROOT}/packaging/chocolatey/update-nuspec"
NUSPEC_TEMPLATE="${TEMPLATE_DIR}/update-nuspec.nuspec"

if [[ ! -f "${NUSPEC_TEMPLATE}" ]]; then
  echo "Chocolatey template not found: ${NUSPEC_TEMPLATE}" >&2
  exit 1
fi

STAGING="${OUTPUT_DIR}/staging"
rm -rf "${STAGING}"
mkdir -p "${STAGING}/tools" "${OUTPUT_DIR}"

cp "${NUSPEC_TEMPLATE}" "${STAGING}/update-nuspec.nuspec"
cp "${TEMPLATE_DIR}/tools/"*.ps1 "${STAGING}/tools/"

unzip -j -o "${WINDOWS_ZIP}" "update-nuspec.exe" -d "${STAGING}/tools/"

if [[ ! -f "${STAGING}/tools/update-nuspec.exe" ]]; then
  echo "update-nuspec.exe not found in ${WINDOWS_ZIP}" >&2
  exit 1
fi

perl -pi -e "s|<version>.*</version>|<version>${VERSION}</version>|" "${STAGING}/update-nuspec.nuspec"

nuget pack "${STAGING}/update-nuspec.nuspec" \
  -Version "${VERSION}" \
  -OutputDirectory "${OUTPUT_DIR}" \
  -NoDefaultExcludes

echo "Packed Chocolatey package: ${OUTPUT_DIR}/update-nuspec.${VERSION}.nupkg"
