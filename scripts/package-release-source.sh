#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <repo-root> <output-dir>" >&2
  exit 1
}

VERSION="${1:-}"
REPO_ROOT="${2:-}"
OUT_DIR="${3:-}"

if [[ -z "${VERSION}" || -z "${REPO_ROOT}" || -z "${OUT_DIR}" ]]; then
  usage
fi

SOURCE_DIR="${REPO_ROOT}/update-nuspec"
if [[ ! -d "${SOURCE_DIR}" ]]; then
  echo "Source directory not found: ${SOURCE_DIR}" >&2
  exit 1
fi

ARCHIVE="${OUT_DIR}/update-nuspec-${VERSION}-src.tar.gz"
mkdir -p "${OUT_DIR}"
rm -f "${ARCHIVE}"
git -C "${REPO_ROOT}" archive --format=tar.gz --prefix=update-nuspec/ HEAD:update-nuspec -o "${ARCHIVE}"
echo "${ARCHIVE}"
