# Packaging

Standalone CLI distribution for `update-nuspec` (outside Docker / Azure DevOps).

## GitHub Release assets

On each push to `master`, `release/*`, or `hotfix/*`, the `publish-and-package` job builds and publishes (preview branches produce a GitHub **Pre-release** and a prerelease Chocolatey package; Homebrew is master-only):

| Asset | Platform |
|-------|----------|
| `update-nuspec-{version}-x86_64-unknown-linux-musl.tar.gz` | Linux x64 (static musl) |
| `update-nuspec-{version}-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `update-nuspec-{version}-x86_64-apple-darwin.tar.gz` | macOS Intel |
| `update-nuspec-{version}-x86_64-pc-windows-msvc.zip` | Windows x64 |
| `SHA256SUMS` | Checksums for binary archives |
| `*.vsix` | Azure DevOps extension |

Binaries are built once in the `release-binaries` matrix; `build` reuses `ado-binary-*` for Docker and ADO.

## Homebrew (homebrew-core)

Target install command:

```bash
brew install update-nuspec
```

That works only after the formula is merged into [Homebrew/homebrew-core](https://github.com/Homebrew/homebrew-core) as `Formula/u/update-nuspec.rb`.

### CI automation (each `master` release)

| Step | What happens |
|------|----------------|
| `update-homebrew-core-formula.sh` | Generates formula draft in CI workspace (`packaging/homebrew-core/`, not committed) |
| Detect formula in core | HTTP check on `Formula/u/update-nuspec.rb` in homebrew-core |
| `publish-homebrew-core-pr.sh` | **If not in core:** push to `denis-peshkov/homebrew-core:update-nuspec`, open upstream PR |
| `brew bump-formula-pr` | **If in core:** open version-bump PR (needs `HOMEBREW_GITHUB_API_KEY`) |
| `stage-chocolatey-package.sh` | Embeds Windows `update-nuspec.exe` from release zip into `.nupkg`; optional push via `CHOCOLATEY_API_KEY` |

### Secrets

| Secret | Purpose |
|--------|---------|
| `TAGTOKEN` | Push git tags and `action.yml` pins in `build`; fallback for `homebrew-core` fork push / initial PR (`repo` scope) |
| `HOMEBREW_GITHUB_API_KEY` | [PAT](https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request#generating-a-personal-access-token-classic) with `public_repo` for `brew bump-formula-pr` after formula is in core |
| `CHOCOLATEY_API_KEY` | API key for pushing the package to chocolatey.org |

Local test before the first PR:

```bash
brew install --build-from-source ./packaging/homebrew-core/update-nuspec.rb
update-nuspec --version
```

## Chocolatey

Package source: [`packaging/chocolatey/update-nuspec/`](chocolatey/update-nuspec/).

CI embeds `update-nuspec.exe` from the Windows release zip (`release-binaries` matrix artifact) into the `.nupkg` — no remote download or checksum in `chocolateyinstall.ps1`.

### Local test

Build or download the Windows zip, then stage and pack:

```bash
./scripts/stage-chocolatey-package.sh 1.2.3 dist/update-nuspec-1.2.3-x86_64-pc-windows-msvc.zip . dist/choco
choco install update-nuspec -s dist/choco --force
```

### CI publish (optional)

Set repository secret `CHOCOLATEY_API_KEY` to push to chocolatey.org on release.

### chocolatey.org community

To publish publicly, open a PR to [chocolatey-community/chocolatey-packages](https://github.com/chocolatey-community/chocolatey-packages) using the generated files from this directory.

## Scripts

| Script | Purpose |
|--------|---------|
| [`scripts/package-release-binary.sh`](../scripts/package-release-binary.sh) | Build `.tar.gz` / `.zip` from a compiled binary |
| [`scripts/pin-action-image.sh`](../scripts/pin-action-image.sh) | Pin `action.yml` to GHCR image tag per git release tag |
| [`scripts/update-homebrew-core-formula.sh`](../scripts/update-homebrew-core-formula.sh) | Regenerate homebrew-core formula draft from source tarball `sha256` |
| [`scripts/publish-homebrew-core-pr.sh`](../scripts/publish-homebrew-core-pr.sh) | Push formula to `denis-peshkov/homebrew-core` and open upstream PR |
| [`scripts/stage-chocolatey-package.sh`](../scripts/stage-chocolatey-package.sh) | Stage Chocolatey package with embedded Windows exe and `nuget pack` |

Manual bump after a release:

```bash
curl -fsSL -o dist/source.tar.gz "https://github.com/denis-peshkov/update-nuspec-action/archive/refs/tags/v1.2.3.tar.gz"
sha256sum dist/source.tar.gz
./scripts/update-homebrew-core-formula.sh 1.2.3 <sha256> .
./scripts/stage-chocolatey-package.sh 1.2.3 dist/update-nuspec-1.2.3-x86_64-pc-windows-msvc.zip . dist/choco
```
