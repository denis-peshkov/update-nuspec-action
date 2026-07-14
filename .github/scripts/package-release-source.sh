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

WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

git -C "${REPO_ROOT}" archive --format=tar --prefix=update-nuspec/ HEAD:update-nuspec \
  | tar -x -C "${WORK}"

perl -pi -e "s/^version = .*/version = \"${VERSION}\"/" "${WORK}/update-nuspec/Cargo.toml"

tar -czf "${ARCHIVE}" -C "${WORK}" update-nuspec
echo "${ARCHIVE}"
