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
FORMULA_SRC="${REPO_ROOT}/distribution/homebrew-core/update-nuspec.rb"
FORMULA_DST="Formula/u/update-nuspec.rb"
PR_HEAD="denis-peshkov:${BRANCH}"
PR_COMPARE_URL="https://github.com/${UPSTREAM_REPO}/compare/${UPSTREAM_DEFAULT_BRANCH}...${PR_HEAD}?expand=1"

if [[ -z "${REPO_ROOT}" || -z "${GIT_TOKEN}" || -z "${VERSION}" ]]; then
  usage
fi

if [[ ! -f "${FORMULA_SRC}" ]]; then
  echo "Formula draft not found: ${FORMULA_SRC}" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI (gh) is required to open a Homebrew PR" >&2
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
echo "Published formula to ${CORE_REPO} branch ${BRANCH}"

PR_BODY="$(cat <<'EOF'
-----

<!-- If your PR is a formula addition/update, please rename the PR to `update-nuspec VERSION (new formula)` and ensure it passes all checks.
`update-nuspec` is the name of the formula you're editing. -->

- [x] Have you followed the [guidelines for contributing](https://github.com/Homebrew/homebrew-core/blob/HEAD/CONTRIBUTING.md)?
- [x] Have you ensured that your commits follow the [commit style guide](https://docs.brew.sh/Formula-Cookbook#commit)?
- [x] Have you checked that there aren't other open [pull requests](https://github.com/Homebrew/homebrew-core/pulls) for the same formula update/change?
- [x] Have you built your formula locally with `HOMEBREW_NO_INSTALL_FROM_API=1 brew install --build-from-source update-nuspec`?
- [x] Is your test running fine `brew test update-nuspec`?
- [x] Does your build pass `brew audit --strict update-nuspec` (after doing `HOMEBREW_NO_INSTALL_FROM_API=1 brew install --build-from-source update-nuspec`)? If this is a new formula, does it pass `brew audit --new update-nuspec`?

Automated release CI (`publish-homebrew`) verified the formula draft from the GitHub Release source archive before opening this PR.

```bash
brew install update-nuspec
update-nuspec --version
```

-----

- [ ] AI was used to generate or assist with generating this PR. *Please specify below how you used AI to help you, and what steps you have taken to manually verify the changes*.

-----
EOF
)"
PR_TITLE="update-nuspec ${VERSION} (new formula)"

open_pr_with_gh() {
  local token="$1"
  export GH_TOKEN="${token}"
  gh auth setup-git >/dev/null 2>&1 || true
  gh pr create \
    --repo "${UPSTREAM_REPO}" \
    --head "${PR_HEAD}" \
    --base "${UPSTREAM_DEFAULT_BRANCH}" \
    --title "${PR_TITLE}" \
    --body "${PR_BODY}"
}

open_pr_with_rest() {
  local token="$1"
  local payload
  payload="$(jq -n \
    --arg title "${PR_TITLE}" \
    --arg head "${PR_HEAD}" \
    --arg base "${UPSTREAM_DEFAULT_BRANCH}" \
    --arg body "${PR_BODY}" \
    '{title: $title, head: $head, base: $base, body: $body}')"
  curl -fsSL -X POST \
    -H "Authorization: Bearer ${token}" \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "https://api.github.com/repos/${UPSTREAM_REPO}/pulls" \
    -d "${payload}"
}

try_open_pr() {
  local token="$1"
  export GH_TOKEN="${token}"

  if open_pr_with_gh "${token}" 2>/dev/null; then
    return 0
  fi

  if open_pr_with_rest "${token}" >/dev/null 2>&1; then
    return 0
  fi

  return 1
}

print_token_help() {
  cat >&2 <<EOF
Failed to open a pull request to ${UPSTREAM_REPO}.

Fork branch was pushed: ${CORE_REPO}:${BRANCH}
Manual compare URL: ${PR_COMPARE_URL}

HOMEBREW_GITHUB_API_KEY (or TAGTOKEN fallback) must be a classic PAT with the
public_repo scope, or a fine-grained PAT with Pull requests: Read and write on
${UPSTREAM_REPO}. See:
https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request#generating-a-personal-access-token-classic
EOF
}

export GH_TOKEN="${GH_API_TOKEN}"
open_pr_count="$(gh pr list \
  --repo "${UPSTREAM_REPO}" \
  --head "${PR_HEAD}" \
  --state open \
  --json number \
  --jq 'length')"

if [[ "${open_pr_count}" != "0" ]]; then
  existing_url="$(gh pr list \
    --repo "${UPSTREAM_REPO}" \
    --head "${PR_HEAD}" \
    --state open \
    --json url \
    --jq '.[0].url')"
  echo "Open PR already exists: ${existing_url}"
  exit 0
fi

tokens_to_try=()
if [[ -n "${GH_API_TOKEN}" ]]; then
  tokens_to_try+=("${GH_API_TOKEN}")
fi
if [[ -n "${GIT_TOKEN}" && "${GIT_TOKEN}" != "${GH_API_TOKEN}" ]]; then
  tokens_to_try+=("${GIT_TOKEN}")
fi

for token in "${tokens_to_try[@]}"; do
  if try_open_pr "${token}"; then
    echo "Opened PR to ${UPSTREAM_REPO}"
    gh pr list --repo "${UPSTREAM_REPO}" --head "${PR_HEAD}" --state open --json url --jq '.[0].url'
    exit 0
  fi
done

print_token_help
exit 1
