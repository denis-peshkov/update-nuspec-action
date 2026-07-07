#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <repo-root> <git-token> <version> [github-api-token]" >&2
  exit 1
}

REPO_ROOT="${1:-}"
GIT_TOKEN="${2:-}"
VERSION="${3:-}"
GH_API_TOKEN="${4:-${GIT_TOKEN}}"

CORE_REPO="denis-peshkov/homebrew-core"
UPSTREAM_REPO="Homebrew/homebrew-core"
UPSTREAM_DEFAULT_BRANCH="main"
CORE_URL="https://x-access-token:${GIT_TOKEN}@github.com/${CORE_REPO}.git"
BRANCH="update-nuspec"
FORMULA_SRC="${REPO_ROOT}/packaging/homebrew-core/update-nuspec.rb"
FORMULA_DST="Formula/u/update-nuspec.rb"

if [[ -z "${REPO_ROOT}" || -z "${GIT_TOKEN}" || -z "${VERSION}" ]]; then
  usage
fi

if [[ ! -f "${FORMULA_SRC}" ]]; then
  echo "Formula draft not found: ${FORMULA_SRC}" >&2
  exit 1
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT

if git clone --depth 1 --branch "${UPSTREAM_DEFAULT_BRANCH}" "${CORE_URL}" "${WORK_DIR}/core" 2>/dev/null; then
  :
else
  git clone --depth 1 "${CORE_URL}" "${WORK_DIR}/core"
fi

cd "${WORK_DIR}/core"
git checkout -B "${BRANCH}"
mkdir -p Formula/u
cp "${FORMULA_SRC}" "${FORMULA_DST}"
git add "${FORMULA_DST}"

if git diff --cached --quiet; then
  echo "homebrew-core fork already has current formula"
else
  git -c user.name="Denis Peshkov" -c user.email="denis.peshkov@outlook.com" \
    commit -m "update-nuspec ${VERSION} (new formula)"
fi

git push -u origin "${BRANCH}" --force

if command -v gh >/dev/null 2>&1; then
  export GH_TOKEN="${GH_API_TOKEN}"
  PR_URL="https://github.com/${UPSTREAM_REPO}/compare/${UPSTREAM_DEFAULT_BRANCH}...denis-peshkov:${BRANCH}?expand=1"
  if gh pr list --repo "${UPSTREAM_REPO}" --head "denis-peshkov:${BRANCH}" --state open --json number --jq 'length' | grep -qx '0'; then
    if ! gh pr create \
      --repo "${UPSTREAM_REPO}" \
      --head "denis-peshkov:${BRANCH}" \
      --base "${UPSTREAM_DEFAULT_BRANCH}" \
      --title "update-nuspec ${VERSION} (new formula)" \
      --body "$(cat <<EOF
- [x] Have you followed the [guidelines for contributing](https://github.com/Homebrew/homebrew-core/blob/master/CONTRIBUTING.md)?
- [x] Have you ensured that your commits follow the [commit style guide](https://docs.brew.sh/Formula-Cookbook#commit)?

\`\`\`bash
brew install update-nuspec
update-nuspec --version
\`\`\`
EOF
)"; then
      echo "gh pr create failed; formula is on fork — open PR manually:" >&2
      echo "${PR_URL}" >&2
    else
      echo "Opened PR to ${UPSTREAM_REPO}"
    fi
  else
    echo "Open PR already exists for ${BRANCH}"
  fi
else
  echo "gh CLI not found; formula pushed to ${CORE_REPO}:${BRANCH}"
  echo "Open PR manually: https://github.com/${UPSTREAM_REPO}/compare/${UPSTREAM_DEFAULT_BRANCH}...denis-peshkov:${BRANCH}?expand=1"
fi

echo "Published formula to ${CORE_REPO} branch ${BRANCH}"
