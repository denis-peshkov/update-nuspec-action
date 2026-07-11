#!/usr/bin/env bash
# Resolve GHCR image tag for update-nuspec-action.
# 1) explicit imageTag input wins
# 2) else derive from github.action_ref (@v2.0.117 → 2.0.117, @master → latest)
set -euo pipefail

EXPLICIT="${1:-}"
ACTION_REF="${2:-}"

if [[ -n "${EXPLICIT}" ]]; then
  echo "${EXPLICIT}"
  exit 0
fi

ref="${ACTION_REF}"

case "${ref}" in
  refs/heads/master | master | refs/heads/main | main)
    echo "latest"
    exit 0
    ;;
esac

ref="${ref#refs/tags/}"
ref="${ref#refs/heads/}"
ref="${ref#v}"

if [[ -z "${ref}" ]]; then
  echo "latest"
else
  echo "${ref}"
fi
