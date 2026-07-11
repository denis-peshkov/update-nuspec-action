#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version>" >&2
  exit 1
}

VERSION="${1:-}"
if [[ -z "${VERSION}" ]]; then
  usage
fi

echo "https://github.com/denis-peshkov/update-nuspec-action/releases/download/v${VERSION}/update-nuspec-${VERSION}-src.tar.gz"
