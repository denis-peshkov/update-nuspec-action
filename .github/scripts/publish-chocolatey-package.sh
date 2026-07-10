#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version> <nupkg-path>" >&2
  exit 1
}

VERSION="${1:-}"
NUPKG="${2:-}"
PACKAGE_ID="update-nuspec"
PUSH_SOURCE="https://push.chocolatey.org/api/v2/package"
PACKAGE_URL="https://community.chocolatey.org/packages/${PACKAGE_ID}"

if [[ -z "${VERSION}" || -z "${NUPKG}" ]]; then
  usage
fi

if [[ ! -f "${NUPKG}" ]]; then
  echo "Package not found: ${NUPKG}" >&2
  exit 1
fi

if [[ -z "${CHOCOLATEY_API_KEY:-}" ]]; then
  echo "CHOCOLATEY_API_KEY is required" >&2
  exit 1
fi

package_submitted_status() {
  local check_version="$1"
  local url="https://community.chocolatey.org/api/v2/Packages(Id='${PACKAGE_ID}',Version='${check_version}')"
  curl -fsSL "${url}" 2>/dev/null \
    | sed -n 's#.*<d:PackageSubmittedStatus>\([^<]*\)</d:PackageSubmittedStatus>.*#\1#p' \
    | head -n 1
}

find_pending_version() {
  local target_version="$1"

  if [[ ! "${target_version}" =~ ^([0-9]+\.[0-9]+\.)([0-9]+)(.*)$ ]]; then
    return 1
  fi

  local prefix="${BASH_REMATCH[1]}"
  local run_num="${BASH_REMATCH[2]}"
  local suffix="${BASH_REMATCH[3]}"
  local i check_version status

  for (( i=run_num-1; i>=run_num-10 && i>=0; i-- )); do
    check_version="${prefix}${i}${suffix}"
    status="$(package_submitted_status "${check_version}")"
    if [[ "${status}" == "Pending" ]]; then
      echo "${check_version}"
      return 0
    fi
  done

  return 1
}

report_moderation_block() {
  local pending_version="$1"
  echo "::error title=Chocolatey moderation queue::Push failed: ${PACKAGE_ID} ${pending_version} is still in moderation (PackageSubmittedStatus=Pending). chocolatey.org returns HTTP 403 for the next version until the previous one is approved. Open ${PACKAGE_URL}/${pending_version} and re-run this job after approval." >&2
}

PENDING_VERSION="$(find_pending_version "${VERSION}" || true)"
if [[ -n "${PENDING_VERSION}" ]]; then
  report_moderation_block "${PENDING_VERSION}"
  exit 1
fi

PUSH_LOG="$(mktemp)"
trap 'rm -f "${PUSH_LOG}"' EXIT

if nuget push "${NUPKG}" \
  -ApiKey "${CHOCOLATEY_API_KEY}" \
  -Source "${PUSH_SOURCE}" \
  -SkipDuplicate 2>&1 | tee "${PUSH_LOG}"; then
  exit 0
fi

if grep -q '403' "${PUSH_LOG}" || grep -qi 'Forbidden' "${PUSH_LOG}"; then
  PENDING_VERSION="$(find_pending_version "${VERSION}" || true)"
  if [[ -n "${PENDING_VERSION}" ]]; then
    report_moderation_block "${PENDING_VERSION}"
    exit 1
  fi

  echo "::error title=Chocolatey push forbidden (HTTP 403)::nuget push was rejected by chocolatey.org. This is not HTTP 409 (duplicate version). Common causes: invalid CHOCOLATEY_API_KEY, not a package maintainer, or a previous version still in moderation. Check ${PACKAGE_URL}#versionhistory" >&2
  exit 1
fi

echo "Chocolatey push failed:"
cat "${PUSH_LOG}" >&2
exit 1
