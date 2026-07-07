#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <target-triple> <binary-path> <output-dir>" >&2
  exit 1
}

VERSION="${1:-}"
TARGET="${2:-}"
BINARY="${3:-}"
OUT_DIR="${4:-}"

if [[ -z "${VERSION}" || -z "${TARGET}" || -z "${BINARY}" || -z "${OUT_DIR}" ]]; then
  usage
fi

if [[ ! -f "${BINARY}" ]]; then
  echo "Binary not found: ${BINARY}" >&2
  exit 1
fi

ARCHIVE_BASE="update-nuspec-${VERSION}-${TARGET}"
STAGING="${OUT_DIR}/staging-${TARGET}"
rm -rf "${STAGING}"
mkdir -p "${STAGING}" "${OUT_DIR}"

BINARY_NAME="$(basename "${BINARY}")"
cp "${BINARY}" "${STAGING}/${BINARY_NAME}"

if [[ "${TARGET}" == *"windows"* ]]; then
  ARCHIVE="${OUT_DIR}/${ARCHIVE_BASE}.zip"
  rm -f "${ARCHIVE}"
  (
    cd "${STAGING}"
    if command -v zip >/dev/null 2>&1; then
      zip -q "${ARCHIVE}" "${BINARY_NAME}"
    else
      powershell -NoProfile -Command "Compress-Archive -Path '${BINARY_NAME}' -DestinationPath '${ARCHIVE}' -Force"
    fi
  )
else
  ARCHIVE="${OUT_DIR}/${ARCHIVE_BASE}.tar.gz"
  rm -f "${ARCHIVE}"
  tar -czf "${ARCHIVE}" -C "${STAGING}" "${BINARY_NAME}"
fi

rm -rf "${STAGING}"
echo "${ARCHIVE}"
