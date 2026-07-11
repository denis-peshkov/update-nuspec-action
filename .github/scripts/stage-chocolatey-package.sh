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

TEMPLATE_DIR="${REPO_ROOT}/distribution/chocolatey/update-nuspec"
NUSPEC_TEMPLATE="${TEMPLATE_DIR}/update-nuspec.nuspec"
LICENSE_TXT="${TEMPLATE_DIR}/tools/LICENSE.txt"
VERIFICATION_TEMPLATE="${TEMPLATE_DIR}/tools/VERIFICATION.txt"

if [[ ! -f "${NUSPEC_TEMPLATE}" ]]; then
  echo "Chocolatey template not found: ${NUSPEC_TEMPLATE}" >&2
  exit 1
fi

if [[ ! -f "${LICENSE_TXT}" ]]; then
  echo "LICENSE.txt not found: ${LICENSE_TXT}" >&2
  exit 1
fi

if [[ ! -f "${VERIFICATION_TEMPLATE}" ]]; then
  echo "VERIFICATION.txt template not found: ${VERIFICATION_TEMPLATE}" >&2
  exit 1
fi

STAGING="${OUTPUT_DIR}/staging"
rm -rf "${STAGING}"
mkdir -p "${STAGING}/tools" "${OUTPUT_DIR}"

cp "${NUSPEC_TEMPLATE}" "${STAGING}/update-nuspec.nuspec"
cp "${TEMPLATE_DIR}/tools/"*.ps1 "${STAGING}/tools/"
cp "${LICENSE_TXT}" "${STAGING}/tools/LICENSE.txt"

unzip -j -o "${WINDOWS_ZIP}" "update-nuspec.exe" -d "${STAGING}/tools/"

if [[ ! -f "${STAGING}/tools/update-nuspec.exe" ]]; then
  echo "update-nuspec.exe not found in ${WINDOWS_ZIP}" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  EXE_SHA256="$(sha256sum "${STAGING}/tools/update-nuspec.exe" | awk '{print toupper($1)}')"
else
  EXE_SHA256="$(shasum -a 256 "${STAGING}/tools/update-nuspec.exe" | awk '{print toupper($1)}')"
fi

RELEASE_URL="https://github.com/denis-peshkov/update-nuspec-action/releases/download/v${VERSION}/update-nuspec-${VERSION}-x86_64-pc-windows-msvc.zip"

perl -pe "s|__RELEASE_URL__|${RELEASE_URL}|g; s|__EXE_SHA256__|${EXE_SHA256}|g" \
  "${VERIFICATION_TEMPLATE}" > "${STAGING}/tools/VERIFICATION.txt"

perl -pi -e "s|<version>.*</version>|<version>${VERSION}</version>|" "${STAGING}/update-nuspec.nuspec"

if ! command -v choco >/dev/null 2>&1; then
  echo "Chocolatey CLI (choco) is required to pack packages with Chocolatey nuspec metadata." >&2
  echo "Run .github/scripts/setup-chocolatey-cli.sh first, or install choco locally." >&2
  exit 1
fi

choco pack "${STAGING}/update-nuspec.nuspec" \
  --version="${VERSION}" \
  --outputdirectory="${OUTPUT_DIR}"

echo "Packed Chocolatey package: ${OUTPUT_DIR}/update-nuspec.${VERSION}.nupkg"
