#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <major> <minor>" >&2
  exit 1
}

VERSION="${1:-}"
MAJOR="${2:-}"
MINOR="${3:-}"

if [[ -z "${VERSION}" || -z "${MAJOR}" || -z "${MINOR}" ]]; then
  usage
fi

if [[ -z "${TAGTOKEN:-}" ]]; then
  echo "TAGTOKEN is required" >&2
  exit 1
fi

git config user.name 'Denis Peshkov'
git config user.email 'denis.peshkov@outlook.com'
git remote set-url origin "https://x-access-token:${TAGTOKEN}@github.com/${GITHUB_REPOSITORY}"

commit="$(git rev-parse HEAD)"

git tag -f "v${VERSION}" "${commit}"
git tag -f "v${MAJOR}.${MINOR}" "${commit}"
git tag -f "v${MAJOR}" "${commit}"
git push origin "v${VERSION}" "v${MAJOR}.${MINOR}" "v${MAJOR}" -f

echo "Pushed tags v${VERSION}, v${MAJOR}.${MINOR}, and v${MAJOR} at ${commit}" >&2
echo "${commit}"
