# Packaging

Standalone CLI distribution for `update-nuspec` (outside Docker / Azure DevOps).

## CI actions

Orchestrator: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

On each **push** to `master`, `release/*`, or `hotfix/*`, three publish jobs run **in parallel** after the `build` action completes (preview branches produce a GitHub **Pre-release** and a prerelease Chocolatey package; Homebrew is master-only):

| Composite action | What it publishes |
|------------------|-------------------|
| [`publish-github-release`](../.github/actions/publish-github-release/action.yml) | GitHub Release assets (see below) |
| [`publish-chocolatey`](../.github/actions/publish-chocolatey/action.yml) | chocolatey.org `.nupkg` (embedded Windows exe) |
| [`publish-homebrew`](../.github/actions/publish-homebrew/action.yml) | homebrew-core formula PR / bump (`master` only) |

Upstream jobs (same pipeline run):

| Composite action | Role |
|------------------|------|
| [`version`](../.github/actions/version/action.yml) | GitVersion |
| [`release-binary`](../.github/actions/release-binary/action.yml) | Matrix build (4 targets in `ci.yml`); `release-binary-*` artifacts for publish |
| [`build`](../.github/actions/build/action.yml) | Tests, GHCR, ADO VSIX (`ado-extension-vsix` artifact) |

Binaries are built once in the `release-binary` matrix; `build` reuses `ado-binary-*` for Docker and ADO.

## GitHub Release assets

Published by `publish-github-release` action:

| Asset | Platform |
|-------|----------|
| `update-nuspec-{version}-x86_64-unknown-linux-musl.tar.gz` | Linux x64 (static musl) |
| `update-nuspec-{version}-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `update-nuspec-{version}-x86_64-apple-darwin.tar.gz` | macOS Intel |
| `update-nuspec-{version}-x86_64-pc-windows-msvc.zip` | Windows x64 |
| `SHA256SUMS` | Checksums for binary archives |
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
| `update-homebrew-core-formula.sh` | Patches formula draft in CI workspace (`packaging/homebrew-core/`, not committed) |
| Detect formula in core | HTTP check on `Formula/u/update-nuspec.rb` in homebrew-core |
| `publish-homebrew-core-pr.sh` | **If not in core:** push to `denis-peshkov/homebrew-core:update-nuspec`, open upstream PR |
| `brew bump-formula-pr` | **If in core:** open version-bump PR (needs `HOMEBREW_GITHUB_API_KEY`) |

### Secrets

| Secret | Purpose |
|--------|---------|
| `TAGTOKEN` | Push git tags and `action.yml` pins in `build` action; push to `homebrew-core` fork for initial PR (`repo` scope) |
| `HOMEBREW_GITHUB_API_KEY` | [PAT](https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request#generating-a-personal-access-token-classic) with `public_repo` for `brew bump-formula-pr` after formula is in core |
| `CHOCOLATEY_API_KEY` | API key for `publish-chocolatey` action → chocolatey.org |

Local test before the first PR:

```bash
brew install --build-from-source ./packaging/homebrew-core/update-nuspec.rb
update-nuspec --version
```

## Chocolatey

Package source: [`packaging/chocolatey/update-nuspec/`](chocolatey/update-nuspec/).

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
| [`scripts/pin-action-image.sh`](../scripts/pin-action-image.sh) | Pin `action.yml` to GHCR image tag per git release tag (`build` action) |
| [`.github/scripts/update-homebrew-core-formula.sh`](../.github/scripts/update-homebrew-core-formula.sh) | Patch homebrew-core formula `url` + `sha256` |
| [`.github/scripts/publish-homebrew-core-pr.sh`](../.github/scripts/publish-homebrew-core-pr.sh) | Push formula to `denis-peshkov/homebrew-core` and open upstream PR |
| [`.github/scripts/publish-chocolatey-package.sh`](../.github/scripts/publish-chocolatey-package.sh) | Push `.nupkg` to chocolatey.org; detect moderation queue via OData |
| [`.github/scripts/stage-chocolatey-package.sh`](../.github/scripts/stage-chocolatey-package.sh) | Stage Chocolatey package with embedded Windows exe and `nuget pack` |

Manual bump after a release:

```bash
curl -fsSL -o dist/source.tar.gz "https://github.com/denis-peshkov/update-nuspec-action/archive/refs/tags/v1.2.3.tar.gz"
sha256sum dist/source.tar.gz
./.github/scripts/update-homebrew-core-formula.sh 1.2.3 <sha256> .
./.github/scripts/stage-chocolatey-package.sh 1.2.3 dist/update-nuspec-1.2.3-x86_64-pc-windows-msvc.zip . dist/choco
```
