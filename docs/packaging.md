# Packaging

Standalone CLI distribution for `update-nuspec` (outside Docker / Azure DevOps).

Pipeline overview: [ci-cd.md](ci-cd.md).

## CI actions

Orchestrator: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

On each **push** to `master`, `release/*`, or `hotfix/*`:

- **`push-tags`** runs after `test` on **`master` only**
- Then in parallel: **`publish-github-action`**, **`publish-ado-extension`**, **`publish-chocolatey`**, **`publish-homebrew`** (master)
- **`publish-github-release`** runs after **`publish-ado-extension`** (master only)

| Composite action | What it publishes |
|------------------|-------------------|
| [`push-tags`](../.github/actions/push-tags/action.yml) | Push git tags (`master` only) |
| [`publish-github-release`](../.github/actions/publish-github-release/action.yml) | GitHub Release assets (`master` only; after `publish-ado-extension`) |
| [`publish-chocolatey`](../.github/actions/publish-chocolatey/action.yml) | chocolatey.org `.nupkg` (embedded Windows exe) |
| [`publish-homebrew`](../.github/actions/publish-homebrew/action.yml) | homebrew-core formula PR / bump (`master` only) |

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
| `update-homebrew-core-formula.sh` | Patches formula draft in CI workspace (`packaging/homebrew-core/`, not committed) |
| Detect formula in core | HTTP check on `Formula/u/update-nuspec.rb` in homebrew-core |
| `publish-homebrew-core-pr.sh` | **If not in core:** push to `denis-peshkov/homebrew-core:update-nuspec`, open upstream PR |
| `brew bump-formula-pr` | **If in core:** open version-bump PR (needs `HOMEBREW_GITHUB_API_KEY`) |

### Secrets

| Secret | Purpose |
|--------|---------|
| `TAGTOKEN` | Moving git tags in `push-tags`; Homebrew fork push / initial PR (`repo` scope) |
| `HOMEBREW_GITHUB_API_KEY` | [PAT](https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request#generating-a-personal-access-token-classic) with `public_repo` for `brew bump-formula-pr` after formula is in core |
| `CHOCOLATEY_API_KEY` | API key for `publish-chocolatey` action → chocolatey.org |

Local test before the first PR:

```bash
brew install --build-from-source ./packaging/homebrew-core/update-nuspec.rb
update-nuspec --version
```

## Chocolatey

Package source: [`packaging/chocolatey/update-nuspec/`](../packaging/chocolatey/update-nuspec/).

The `publish-chocolatey` action embeds `update-nuspec.exe` from the Windows release zip (`release-binary` matrix artifact) into the `.nupkg` — no remote download or checksum in `chocolateyinstall.ps1`.

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

| Script | Purpose |
|--------|---------|
| [`scripts/package-release-binary.sh`](../scripts/package-release-binary.sh) | Build `.tar.gz` / `.zip` from a compiled binary (`release-binary` action) |
| [`scripts/package-release-source.sh`](../scripts/package-release-source.sh) | Build `update-nuspec-{version}-src.tar.gz` for Release and Homebrew (`git archive`) |
| [`.github/scripts/release-source-url.sh`](../.github/scripts/release-source-url.sh) | Release download URL for the source archive |
| [`scripts/resolve-action-image-tag.sh`](../scripts/resolve-action-image-tag.sh) | Map action `@ref` / `imageTag` input → GHCR tag (`action.yml`) |
| [`.github/scripts/push-release-git-tags.sh`](../.github/scripts/push-release-git-tags.sh) | Push git tags (`push-tags` action) |
| [`.github/scripts/update-homebrew-core-formula.sh`](../.github/scripts/update-homebrew-core-formula.sh) | Patch homebrew-core formula `url` + `sha256` |
| [`.github/scripts/publish-homebrew-core-pr.sh`](../.github/scripts/publish-homebrew-core-pr.sh) | Push formula to `denis-peshkov/homebrew-core` and open upstream PR |
| [`.github/scripts/publish-chocolatey-package.sh`](../.github/scripts/publish-chocolatey-package.sh) | Push `.nupkg` to chocolatey.org; detect moderation queue via OData |
| [`.github/scripts/stage-chocolatey-package.sh`](../.github/scripts/stage-chocolatey-package.sh) | Stage Chocolatey package with embedded Windows exe and `nuget pack` |

Manual bump after a release:

```bash
./scripts/package-release-source.sh 1.2.3 . dist
sha256sum dist/update-nuspec-1.2.3-src.tar.gz
./.github/scripts/update-homebrew-core-formula.sh 1.2.3 <sha256> .
./.github/scripts/stage-chocolatey-package.sh 1.2.3 dist/update-nuspec-1.2.3-x86_64-pc-windows-msvc.zip . dist/choco
```
