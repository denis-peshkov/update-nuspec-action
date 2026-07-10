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

FORMULA="${REPO_ROOT}/packaging/homebrew-core/update-nuspec.rb"
if [[ ! -f "${FORMULA}" ]]; then
  echo "Formula template not found: ${FORMULA}" >&2
  exit 1
fi

URL="https://github.com/denis-peshkov/update-nuspec-action/archive/refs/tags/v${VERSION}.tar.gz"

# Only url and sha256 change between releases; the rest stays as committed.
perl -pi -e "s|^  url \".*\"|  url \"${URL}\"|" "${FORMULA}"
perl -pi -e "s|^  sha256 \".*\"|  sha256 \"${SOURCE_SHA256}\"|" "${FORMULA}"

echo "Updated homebrew-core formula for v${VERSION}"
