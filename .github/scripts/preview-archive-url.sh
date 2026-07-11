#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <commit-sha>" >&2
  exit 1
}

COMMIT_SHA="${1:-}"
if [[ -z "${COMMIT_SHA}" ]]; then
  usage
fi

echo "https://github.com/denis-peshkov/update-nuspec-action/archive/${COMMIT_SHA}.tar.gz"
