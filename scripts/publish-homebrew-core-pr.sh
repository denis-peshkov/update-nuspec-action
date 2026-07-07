#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <repo-root> <git-token> <version>" >&2
  exit 1
}

REPO_ROOT="${1:-}"
TOKEN="${2:-}"
VERSION="${3:-}"

CORE_REPO="denis-peshkov/homebrew-core"
UPSTREAM_REPO="Homebrew/homebrew-core"
CORE_URL="https://x-access-token:${TOKEN}@github.com/${CORE_REPO}.git"
BRANCH="update-nuspec"
FORMULA_SRC="${REPO_ROOT}/packaging/homebrew-core/update-nuspec.rb"
FORMULA_DST="Formula/u/update-nuspec.rb"

if [[ -z "${REPO_ROOT}" || -z "${TOKEN}" || -z "${VERSION}" ]]; then
  usage
fi

if [[ ! -f "${FORMULA_SRC}" ]]; then
  echo "Formula draft not found: ${FORMULA_SRC}" >&2
  exit 1
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT

if git clone --depth 1 --branch main "${CORE_URL}" "${WORK_DIR}/core" 2>/dev/null; then
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
  export GH_TOKEN="${TOKEN}"
  if gh pr list --repo "${UPSTREAM_REPO}" --head "denis-peshkov:${BRANCH}" --state open --json number --jq 'length' | grep -qx '0'; then
    gh pr create \
      --repo "${UPSTREAM_REPO}" \
      --head "denis-peshkov:${BRANCH}" \
      --base master \
      --title "update-nuspec ${VERSION} (new formula)" \
      --body "$(cat <<EOF
- [x] Have you followed the [guidelines for contributing](https://github.com/Homebrew/homebrew-core/blob/master/CONTRIBUTING.md)?
- [x] Have you ensured that your commits follow the [commit style guide](https://docs.brew.sh/Formula-Cookbook#commit)?

\`\`\`bash
brew install update-nuspec
update-nuspec --version
\`\`\`
EOF
)"
    echo "Opened PR to ${UPSTREAM_REPO}"
  else
    echo "Open PR already exists for ${BRANCH}"
  fi
else
  echo "gh CLI not found; formula pushed to ${CORE_REPO}:${BRANCH}"
fi

echo "Published formula to ${CORE_REPO} branch ${BRANCH}"
