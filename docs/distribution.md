# Distribution

Standalone CLI distribution for `update-nuspec` outside the root composite `action.yml`: GHCR Docker image ([`distribution/github-action/`](../distribution/github-action/)), Homebrew, Chocolatey, GitHub Release binaries, and the Azure DevOps extension ([`distribution/azure-devops-extension/`](../distribution/azure-devops-extension/)).

Pipeline overview: [ci-cd.md](ci-cd.md).

## CI actions

Orchestrator: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

On each **push** to `master`, `release/*`, or `hotfix/*`:

- **`push-tags`** runs after `test` on **`master` only**
- Then in parallel: **`publish-github-action`**, **`publish-ado-extension`**, **`publish-chocolatey`**, **`publish-homebrew`** (master), **`publish-homebrew-tap`** (release/hotfix)
- **`publish-github-release`** runs after **`publish-ado-extension`** (master only)

| Composite action | What it publishes |
|------------------|-------------------|
| [`push-tags`](../.github/actions/push-tags/action.yml) | Push git tags (`master` only) |
| [`publish-github-release`](../.github/actions/publish-github-release/action.yml) | GitHub Release assets (`master` only; after `publish-ado-extension`) |
| [`publish-chocolatey`](../.github/actions/publish-chocolatey/action.yml) | chocolatey.org `.nupkg` (embedded Windows exe) |
| [`publish-homebrew`](../.github/actions/publish-homebrew/action.yml) | homebrew-core formula PR / bump (`master` only) |
| [`publish-homebrew-tap`](../.github/actions/publish-homebrew-tap/action.yml) | Preview formula on branch `homebrew-preview-tap` (`release/*`, `hotfix/*`; commit SHA, no git tag) |

Upstream jobs (same pipeline run):

| Composite action | Role |
|------------------|------|
| [`version`](../.github/actions/version/action.yml) | GitVersion |
| [`release-binary`](../.github/actions/release-binary/action.yml) | Matrix build (4 targets in `ci.yml`); `release-binary-*` artifacts for publish |
| [`test`](../.github/actions/test/action.yml) | Rust/.NET tests, SonarCloud (after matrix) |
| [`push-tags`](../.github/actions/push-tags/action.yml) | Git tags on `master` (after `test`) |
| [`publish-github-action`](../.github/actions/publish-github-action/action.yml) | GHCR + Docker smoke |
| [`publish-ado-extension`](../.github/actions/publish-ado-extension/action.yml) | VSIX + ADO Marketplace |

After `test`, `push-tags` runs on `master`; then `publish-github-action`, `publish-ado-extension`, `publish-chocolatey`, and `publish-homebrew` run in parallel. `publish-github-release` waits for `publish-ado-extension`.

## GitHub Release assets

Published by `publish-github-release` action:

| Asset | Platform |
|-------|----------|
| `update-nuspec-{version}-src.tar.gz` | Rust crate source (`update-nuspec/` only; Homebrew) |
| `update-nuspec-{version}-x86_64-unknown-linux-musl.tar.gz` | Linux x64 (static musl) |
| `update-nuspec-{version}-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `update-nuspec-{version}-x86_64-apple-darwin.tar.gz` | macOS Intel |
| `update-nuspec-{version}-x86_64-pc-windows-msvc.zip` | Windows x64 |
| `SHA256SUMS` | Checksums for binary archives and source archive |
| `*.vsix` | Azure DevOps extension |

## Homebrew (homebrew-core)

Target install command:

```bash
brew install update-nuspec
```

That works only after the formula is merged into [Homebrew/homebrew-core](https://github.com/Homebrew/homebrew-core) as `Formula/u/update-nuspec.rb`.

### CI automation (`publish-homebrew` action, each `master` release)

| Step | What happens |
|------|----------------|
| `package-release-source.sh` | Build `update-nuspec-{version}-src.tar.gz` via `git archive` (only tracked `update-nuspec/` files) |
| inline in action | Patch formula draft `url` + `sha256` in `distribution/homebrew-core/` (not committed) |
| Detect formula in core | HTTP check on `Formula/u/update-nuspec.rb` in homebrew-core |
| `publish-homebrew-core-pr.sh` | **If not in core:** push to fork, open upstream PR (`gh` + REST fallback; fails CI if PR cannot be created) |
| `brew bump-formula-pr` | **If in core:** open version-bump PR (needs `HOMEBREW_GITHUB_API_KEY`) |

### Secrets

| Secret | Purpose |
|--------|---------|
| `TAGTOKEN` | Moving git tags in `push-tags`; Homebrew fork push / initial PR (`repo` scope) |
| `HOMEBREW_GITHUB_API_KEY` | [Classic PAT](https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request#generating-a-personal-access-token-classic) with **`public_repo`** for `gh pr create`, REST PR API, and `brew bump-formula-pr`. If PR creation fails, CI also retries with `TAGTOKEN`. Without `public_repo` you get `Resource not accessible by personal access token (createPullRequest)`. |
| `CHOCOLATEY_API_KEY` | API key for `publish-chocolatey` action → chocolatey.org |

Local test before the first PR:

```bash
brew install --build-from-source ./distribution/homebrew-core/update-nuspec.rb
update-nuspec --version
```

## Homebrew preview tap (branch `homebrew-preview-tap`)

Preview builds from `release/*` and `hotfix/*` — **no git tag**. CI updates `Formula/update-nuspec-preview.rb` on branch **`homebrew-preview-tap`** only (source archive: `…/archive/{commit-sha}.tar.gz`).

### Install

```bash
brew tap denis-peshkov/update-nuspec https://github.com/denis-peshkov/update-nuspec-action --branch homebrew-preview-tap
brew install update-nuspec-preview
```

### CI automation (`publish-homebrew-tap` action)

| Step | What happens |
|------|----------------|
| inline in action | GitHub archive URL for commit SHA, `curl` + `sha256sum` |
| inline in action | Patch template `distribution/homebrew-preview/update-nuspec-preview.rb` |
| `publish-homebrew-preview-tap.sh` | Push `Formula/update-nuspec-preview.rb` to branch `homebrew-preview-tap` |

Secret: **`TAGTOKEN`** (`repo` scope) — push branch `homebrew-preview-tap`.

Template (not installed directly): [`distribution/homebrew-preview/update-nuspec-preview.rb`](../distribution/homebrew-preview/update-nuspec-preview.rb).

## Azure DevOps extension

Source: [`distribution/azure-devops-extension/`](../distribution/azure-devops-extension/) (`vss-extension.json`, task `UpdateNuspec@1`).

Marketplace listing: [`marketplace/overview.md`](../distribution/azure-devops-extension/marketplace/overview.md).

Built and published by `publish-ado-extension` (VSIX artifact → `publish-github-release`; Marketplace on `master`).

## Chocolatey

Package source: [`distribution/chocolatey/update-nuspec/`](../distribution/chocolatey/update-nuspec/).

The `publish-chocolatey` action embeds `update-nuspec.exe` from the Windows release zip (`release-binary` matrix artifact) into the `.nupkg` — no remote download or checksum in `chocolateyinstall.ps1`.

Embedded binaries require `tools/LICENSE.txt` and a generated `tools/VERIFICATION.txt` (SHA256 + official release URL). If moderation rejects a version, re-upload the **same version** after fixing the package.

### Local test

Build or download the Windows zip, then stage and pack:

```bash
./.github/scripts/stage-chocolatey-package.sh 1.2.3 dist/update-nuspec-1.2.3-x86_64-pc-windows-msvc.zip . dist/choco
choco install update-nuspec -s dist/choco --force
```

### CI publish (optional)

Set repository secret `CHOCOLATEY_API_KEY`. If a previous version is still in moderation, `publish-chocolatey-package.sh` fails the job with an error and the pending version URL (chocolatey.org responds with **HTTP 403**, not 409).

### chocolatey.org community

To publish publicly, open a PR to [chocolatey-community/chocolatey-packages](https://github.com/chocolatey-community/chocolatey-packages) using the generated files from this directory.

## Scripts

Список скриптов: [README.md — Scripts](../README.md#scripts).

Manual bump after a release:

```bash
VERSION=1.2.3
./.github/scripts/package-release-source.sh "${VERSION}" . dist
SHA256="$(sha256sum "dist/update-nuspec-${VERSION}-src.tar.gz" | awk '{print $1}')"
URL="https://github.com/denis-peshkov/update-nuspec-action/releases/download/v${VERSION}/update-nuspec-${VERSION}-src.tar.gz"
FORMULA=distribution/homebrew-core/update-nuspec.rb
perl -pi -e "s|^  url \".*\"|  url \"${URL}\"|" "${FORMULA}"
perl -pi -e "s|^  sha256 \".*\"|  sha256 \"${SHA256}\"|" "${FORMULA}"
./.github/scripts/stage-chocolatey-package.sh "${VERSION}" "dist/update-nuspec-${VERSION}-x86_64-pc-windows-msvc.zip" . dist/choco
```
