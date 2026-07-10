#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <mode> <version> <major> <minor>" >&2
  echo "  mode: master | preview" >&2
  exit 1
}

MODE="${1:-}"
VERSION="${2:-}"
MAJOR="${3:-}"
MINOR="${4:-}"
IMAGE_REPO="ghcr.io/denis-peshkov/update-nuspec"

if [[ -z "${MODE}" || -z "${VERSION}" || -z "${MAJOR}" || -z "${MINOR}" ]]; then
  usage
fi

git config user.name 'Denis Peshkov'
git config user.email 'denis.peshkov@outlook.com'

pin_image() {
  local image_tag="$1"
  perl -pi -e "s|^  image: .*|  image: 'docker://${IMAGE_REPO}:${image_tag}'|" action.yml
}

commit_pin() {
  local image_tag="$1"
  pin_image "${image_tag}"
  git add action.yml
  git commit -m "chore(action): use ghcr image ${image_tag} [skip ci]"
}

if [[ "${MODE}" == "master" ]]; then
  # Each git tag (@vX.Y.Z, @vX.Y, @v1) must point to action.yml with the matching GHCR tag.
  commit_pin "${VERSION}"
  git tag "v${VERSION}" -f

  commit_pin "${MAJOR}.${MINOR}"
  git tag "v${MAJOR}.${MINOR}" -f

  commit_pin "${MAJOR}"
  git tag "v${MAJOR}" -f

  commit_pin "latest"
elif [[ "${MODE}" == "preview" ]]; then
  commit_pin "${VERSION}"
  git tag "v${VERSION}" -f
else
  usage
fi

echo "Pinned action.yml (${MODE}):"
grep "^  image:" action.yml
