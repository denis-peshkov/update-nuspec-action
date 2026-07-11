#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <commit-sha> <source-tarball-sha256> <repo-root>" >&2
  exit 1
}

VERSION="${1:-}"
COMMIT_SHA="${2:-}"
SOURCE_SHA256="${3:-}"
REPO_ROOT="${4:-}"

if [[ -z "${VERSION}" || -z "${COMMIT_SHA}" || -z "${SOURCE_SHA256}" || -z "${REPO_ROOT}" ]]; then
  usage
fi

FORMULA="${REPO_ROOT}/packaging/homebrew-preview/update-nuspec-preview.rb"
if [[ ! -f "${FORMULA}" ]]; then
  echo "Formula template not found: ${FORMULA}" >&2
  exit 1
fi

URL="$("${REPO_ROOT}/.github/scripts/preview-archive-url.sh" "${COMMIT_SHA}")"

perl -pi -e "s|^  version \".*\"|  version \"${VERSION}\"|" "${FORMULA}"
perl -pi -e "s|^  url \".*\"|  url \"${URL}\"|" "${FORMULA}"
perl -pi -e "s|^  sha256 \".*\"|  sha256 \"${SOURCE_SHA256}\"|" "${FORMULA}"

echo "Updated preview tap formula for ${VERSION} (${COMMIT_SHA})"
