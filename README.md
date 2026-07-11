[![License](https://img.shields.io/github/license/denis-peshkov/update-nuspec-action)](LICENSE)
[![GitHub Release Date](https://img.shields.io/github/release-date/denis-peshkov/update-nuspec-action?label=released)](https://github.com/denis-peshkov/update-nuspec-action/releases)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=UpdateNuspecTool&metric=coverage)](https://sonarcloud.io/summary/new_code?id=UpdateNuspecTool)
[![issues](https://img.shields.io/github/issues/denis-peshkov/update-nuspec-action)](https://github.com/denis-peshkov/update-nuspec-action/issues)
[![CI](https://github.com/denis-peshkov/update-nuspec-action/actions/workflows/ci.yml/badge.svg)](https://github.com/denis-peshkov/update-nuspec-action/actions/workflows/ci.yml)

![Size](https://img.shields.io/github/repo-size/denis-peshkov/update-nuspec-action)
[![GitHub contributors](https://img.shields.io/github/contributors/denis-peshkov/update-nuspec-action)](https://github.com/denis-peshkov/update-nuspec-action/contributors)
[![GitHub commits since latest release (by date)](https://img.shields.io/github/commits-since/denis-peshkov/update-nuspec-action/latest?label=new+commits)](https://github.com/denis-peshkov/update-nuspec-action/commits/master)
![Activity](https://img.shields.io/github/commit-activity/w/denis-peshkov/update-nuspec-action)
![Activity](https://img.shields.io/github/commit-activity/m/denis-peshkov/update-nuspec-action)
![Activity](https://img.shields.io/github/commit-activity/y/denis-peshkov/update-nuspec-action)

# update-nuspec-action

CLI (Rust) GitHub Action (Docker) that scans .NET projects in a directory and updates the `<dependencies>` section in matching `*.nuspec` files according to `PackageReference` versions from the related `.csproj` (project name = `<id>` in nuspec metadata). Optionally updates `package.json` version and scoped npm dependencies.

**CI/CD:** [pipeline diagram](docs/ci-cd.md) · [packaging](docs/packaging.md)

## Usage

Pin a [release tag](https://github.com/denis-peshkov/update-nuspec-action/releases) (recommended):

```yaml
- uses: actions/checkout@v4

- uses: denis-peshkov/update-nuspec-action@v1
  with:
    dir: src/MyPackage
```

`dir` is relative to `/github/workspace` (repo root after `checkout`). An absolute path (starting with `/`) is used as-is.

Dry-run (report only, no file writes):

```yaml
- uses: denis-peshkov/update-nuspec-action@v1
  with:
    dir: src/MyPackage
    dryRun: true
```

### Checklist (consumer workflow)

```yaml
jobs:
  update-nuspec:
    runs-on: ubuntu-latest   # linux/amd64; see Requirements
    steps:
      - uses: actions/checkout@v4

      - uses: denis-peshkov/update-nuspec-action@v1
        with:
          dir: src/MyPackage   # explicit folder — not "." unless you want the whole repo
          dryRun: false        # true = preview in logs, no writes
        env:
          CONSOLE_ANSI_COLOR: false   # omit or true for colored log (default in image: true)
```

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `dir` | No | `.` | Root folder to scan **recursively** for `.csproj` / `.nuspec` pairs and (when `packageVersion` is set) `package.json`, relative to `/github/workspace`. Prefer a package path (`src/MyPackage`); `.` scans the entire checkout including nested folders (tests, other packages). |
| `dryRun` | No | `false` | `true` — full report in the log, no file changes (`[DRY RUN]`). |
| `packageVersion` | No | *(empty)* | SemVer for `package.json` `version`. Azure DevOps: `$(GitVersion_SemVer)` after `gitversion/execute`. Env fallback: `PACKAGE_VERSION`, `GitVersion_SemVer`. |
| `dependencyScope` | No | *(empty)* | npm package name prefix to set to `^packageVersion`. Skipped when empty. |
| `imageTag` | No | *(from `@ref`)* | GHCR tag override (`2.0.117`, `latest`, `2.0.118-preview`). Default: `@v2.0.117` → `2.0.117`, `@v2` → `2`, `@master` → `latest`. Use for preview branch refs. |

## Outputs

| Output | Description |
|--------|-------------|
| `packageVersion` | Echo of the `packageVersion` input when it was provided. |

## Behavior

- Recursively looks for `*.nuspec` under `dir` (all subfolders).
- Loads `{id}.csproj` from the **same folder as each** `.nuspec`, where `{id}` is `<metadata><id>`.
- **Flat** nuspec — top-level `<dependency id="..." version="..." />` under `<dependencies>`. Package list is taken for `TargetFramework`, or the first TFM from `TargetFrameworks`.
- **Grouped** nuspec — `<group targetFramework="net8.0">` (and other TFMs). Each group is synced only with packages that apply to that TFM in the csproj:
  - `PropertyGroup Condition="'$(TargetFramework)' == 'net6.0'"` (and similar) for version properties;
  - `PackageReference` with `Version="$(PropertyName)"` resolved per TFM;
  - `Condition` on `PackageReference` / `ItemGroup`, including `or` (for example `'$(TargetFramework)' == 'net6.0' or '$(TargetFramework)' == 'net7.0' or '$(TargetFramework)' == 'net8.0'`).
- Updates versions, adds packages from the csproj, removes dependencies that are not in the csproj for that TFM / flat list.
- Saved dependency order: `Cross.*`, then `*Boilerplate*`, then `*.Api.Contract*`, then the rest (A–Z).
- **Console report:** grouped nuspec — one block per `<group targetFramework="...">`; flat nuspec — single block. Categories: deleted, updated, added, not changed.
- `PrivateAssets="All"` references (for example SourceLink) are not written to nuspec.
- Exits with code `0` if no `.nuspec` files are found (prints `*.nuspec files not found!`).
- Prints an error if `dir` does not exist (`Path '…' is not valid!`).
- **Dry-run** — GitHub Action input `dryRun: true`, or CLI flags `--dry-run` / `-d` / `--demo` (or positional `true`): full report, no file save (`[DRY RUN]` in the log).
- **`package.json`** (optional) — when `packageVersion` is set: updates `"version"` in every `package.json` under `dir` (skips `node_modules`). When `dependencyScope` is also set, aligns matching npm dependencies to `^packageVersion`.

Example multi-TFM project: `UpdateNuspecTool.Tests/TestData/Cross.Messaging.csproj` + `Cross.Messaging.nuspec`.

## Example output

Colored log when `CONSOLE_ANSI_COLOR=true` (default in the action image). In **dry-run** mode (`dryRun: true` / `--dry-run`) the same report is printed, categories are shown in gray, and files are not saved (`[DRY RUN]` in the log).

### 1. Sync nuspec dependencies

For each `.nuspec` the tool prints a categorized diff against the sibling `{id}.csproj`:

| Category | Color | Meaning |
|----------|-------|---------|
| Deleted references | red | In nuspec, not in csproj for this TFM |
| Updated references | yellow | Same package id, version changed (`old -> new`) |
| Added references | green | In csproj, missing from nuspec |
| Not changed references | gray | Same id and version |

`Cross.Identity` (`config.nuspec` + `Cross.Identity.csproj`):

```yaml
- uses: denis-peshkov/update-nuspec-action@v1
  with:
    dir: Cross.Identity
```

![GitHub Actions log — Cross.Identity](docs/examples/github-actions-cross-identity.png)

### 2. Update package.json (built npm package)

With `packageVersion` and `dependencyScope` — updates `version` and scoped npm dependencies (e.g. `client/dist/ui/package.json`). After [GitVersion execute](https://github.com/gittools/actions):

```yaml
- uses: gittools/actions/gitversion/execute@v1.1.1
  id: gitversion

- uses: denis-peshkov/update-nuspec-action@v1
  with:
    dir: client/dist/my-app
    packageVersion: ${{ env.GitVersion_SemVer }}
    dependencyScope: '@guru/'              # optional; empty = skip dependency alignment
```

## Requirements

This action is a **Docker container action** (`runs.using: docker` in `action.yml`). GitHub runs it only on **Linux** runners; the image is `linux/amd64` with a `linux-x64` tool binary.

- **Runner:** `ubuntu-latest` (recommended) or any **linux/amd64** self-hosted host with Docker.
- **`windows-latest` / `macos-latest`:** **not supported** — container actions do not run on Windows or macOS hosted runners. Use a separate job on `ubuntu-latest` (other jobs in the workflow may still use Windows).
- **Self-hosted ARM runners:** not supported as-is — use `ubuntu-latest`, or a self-hosted **amd64** Linux agent, or dedicate one job to `runs-on: ubuntu-latest`.
- **Colored log output:** enabled by default in the image (`CONSOLE_ANSI_COLOR=true`). Override with `env: CONSOLE_ANSI_COLOR: false` on the step if needed.

**On Windows:** use the Rust CLI (`cargo build --release` in `update-nuspec/`, see [CLI (local)](#cli-local)) or the [Azure DevOps extension](#azure-devops-extension) (`UpdateNuspec@1` on `windows-latest`).

Mixed workflow example (build on Windows, nuspec sync on Linux):

```yaml
jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      # …

  sync-nuspec:
    runs-on: ubuntu-latest
    steps:
      - uses: denis-peshkov/update-nuspec-action@v1
        with:
          dir: src/MyPackage
```

## Versioning (this repository)

[GitVersion](https://gitversion.net/) (`GitVersion.yml`) on push:

| Branch | SemVer (example) | Git tags | GHCR image | GitHub Release | Chocolatey | Homebrew | ADO extension |
|--------|------------------|----------|------------|----------------|------------|----------|---------------|
| `master` | `1.2.3` (stable) | `v1.2.3`, `v1.2`, `v1` | `:1.2.3`, `:1.2`, `:1`, `:latest` | **Release** (binaries + VSIX) | push (stable) | core PR / bump | Marketplace **public** |
| `release/*`, `hotfix/*` | `1.3.0-preview.4` | — | `:1.3.0-preview.4` only | — | push (prerelease) | preview tap (`homebrew-preview-tap`) | VSIX in CI artifacts |

Preview branches publish GHCR, Chocolatey, and the Homebrew preview tap (branch `homebrew-preview-tap`, no git tags). **Git tags** and **GitHub Release** run on **`master` only** (moving `@v` tags and `:latest` stay on master). Pipeline details: [docs/ci-cd.md](docs/ci-cd.md).

The action is a **composite** wrapper: at runtime it resolves the GHCR tag from the action ref (`@v2.0.117` → `:2.0.117`, `@v2.0` → `:2.0`, `@v2` → `:2`, `@master` → `:latest`) or from optional input `imageTag`.

```yaml
uses: denis-peshkov/update-nuspec-action@v2.0.117   # pulls ghcr.io/.../update-nuspec:2.0.117
uses: denis-peshkov/update-nuspec-action@v2         # pulls :2
uses: denis-peshkov/update-nuspec-action@v2.0       # pulls :2.0

# preview from a branch ref — set imageTag explicitly:
uses: denis-peshkov/update-nuspec-action@release/my-feature
with:
  imageTag: '2.0.118-preview'
  dir: src/MyPackage
```

ADO Marketplace publish uses secret `AZDO_MARKETPLACE_PAT` (scope **Marketplace (Publish)**), publisher **peshkov**.

## Azure DevOps extension

The same tool is available as pipeline task **`UpdateNuspec@1`** on [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=peshkov.update-nuspec). Usage, examples, inputs, and preview install: [azure-devops-extension/marketplace/overview.md](azure-devops-extension/marketplace/overview.md).

## Development

### Repository layout

| Path | Role |
|------|------|
| `update-nuspec/` | Rust CLI and library (`update-nuspec` binary) |
| `UpdateNuspecTool/` | Legacy .NET CLI (parity tests) |
| `UpdateNuspecTool.Tests/` | NUnit tests and fixtures |
| `UpdateNuspecTool.Tests/TestData/` | Sample `.nuspec` / `.csproj` pairs |
| `Dockerfile` | Runtime image (`alpine`): copies prebuilt `musl` binary + `entrypoint.sh` (no Rust build in Docker) |
| `action.yml` | Composite action: resolves GHCR tag from `@ref` or `imageTag`, runs `docker://ghcr.io/denis-peshkov/update-nuspec:…` |
| `update-nuspec-icon.png` | Project icon (repo root) |
| `azure-devops-extension/` | Extension root (`vss-extension.json`); VSIX build in CI (`publish-ado-extension`) |
| `.github/workflows/ci.yml` | CI workflow — see [docs/ci-cd.md](docs/ci-cd.md) |
| `.github/actions/` | Composite CI/CD actions |
| `.github/scripts/` | Publish helper scripts (Homebrew, Chocolatey) |
| `scripts/` | Build helpers (`package-release-binary.sh`, `resolve-action-image-tag.sh`) |
| `packaging/homebrew-preview/` | Preview Homebrew formula template → CI pushes to branch `homebrew-preview-tap` |
| `azure-devops-extension/marketplace/` | Marketplace content: `overview.md`, `license.md`, `extension-icon.png` (symlink to project icon), screenshots |
| `azure-devops-extension/task/` | Pipeline task `UpdateNuspec@1` (TypeScript wrapper + bundled `update-nuspec` binaries) |

### Tests

```bash
cd update-nuspec && cargo test
```

```bash
dotnet restore UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj
dotnet test UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj --configuration Release --no-restore
```

Fixtures: `UpdateNuspecTool.Tests/TestData/` (`MyPackage.nuspec`, `Cross.Messaging.nuspec`, `package.json`, …). In CI the same tests run in the `test` job — see [docs/ci-cd.md](docs/ci-cd.md).

### CLI (local)

**Package managers** (after acceptance in the respective registries; binaries also on [GitHub Releases](https://github.com/denis-peshkov/update-nuspec-action/releases)):

```bash
brew install update-nuspec
```

**Preview** (from `release/*` / `hotfix/*`, branch `homebrew-preview-tap`, no git tag):

```bash
brew tap denis-peshkov/update-nuspec https://github.com/denis-peshkov/update-nuspec-action --branch homebrew-preview-tap
brew install update-nuspec-preview
```

```powershell
choco install update-nuspec
```

First Homebrew submission: [docs/packaging.md](docs/packaging.md#homebrew-homebrew-core).

Options: `--help` / `-h`, `--version` / `-v`, `--dry-run` / `-d` / `--demo` (or positional `true`), `--package-version` / `-pv`, `--dependency-scope` / `-ds`.

```bash
cd update-nuspec
cargo run --bin update-nuspec -- --help
cargo run --bin update-nuspec -- --version
cargo run --bin update-nuspec -- ../UpdateNuspecTool.Tests/TestData --dry-run
cargo run --bin update-nuspec -- ../client/dist/my-app --package-version 1.2.3 --dependency-scope @guru/
```

Release build:

```bash
cd update-nuspec
cargo build --release --bin update-nuspec
./target/release/update-nuspec ../UpdateNuspecTool.Tests/TestData --dry-run
```

**Windows (x64)** — CI builds with `x86_64-pc-windows-msvc` (same binary for Release, Chocolatey, and ADO), or build natively on Windows:

```powershell
cd update-nuspec
cargo build --release --bin update-nuspec
.\target\release\update-nuspec.exe ..\UpdateNuspecTool.Tests\TestData
```

**Linux (x64)** — used in the action Docker image and `ubuntu-latest`:

```bash
cd update-nuspec
cargo build --release --bin update-nuspec
./target/release/update-nuspec ../UpdateNuspecTool.Tests/TestData
```

| Platform | Binary |
|----------|--------|
| Linux x64 | `update-nuspec` |
| Windows x64 | `update-nuspec.exe` |
| macOS (local) | `update-nuspec` (`cargo build --release`) |

### Docker image

The action pulls a **prebuilt image** from GHCR at runtime. CI builds it from the static `musl` binary and pushes tags `X.Y.Z`, `X.Y`, `X`, and `latest` on `master`. The composite `action.yml` maps your `@ref` (or `imageTag` input) to the GHCR tag.

Build locally (stage the binary first, since `Dockerfile` only copies it):

```bash
cd update-nuspec && cargo build --release --target x86_64-unknown-linux-musl --bin update-nuspec && cd ..
mkdir -p docker && cp update-nuspec/target/x86_64-unknown-linux-musl/release/update-nuspec docker/update-nuspec
docker build --platform linux/amd64 -t update-nuspec-action:local .
docker run --rm --platform linux/amd64 \
  -v "$PWD:/github/workspace" \
  update-nuspec-action:local UpdateNuspecTool.Tests/TestData true
```

> First release only: make the GHCR package **public** (`github.com/users/denis-peshkov/packages/container/update-nuspec/settings`) so `uses: …@v1` can pull it without auth.

On Apple Silicon hosts, use `--platform linux/amd64` so the image matches GitHub-hosted runners.

## CI (GitHub Actions)

Workflow: [`.github/workflows/ci.yml`](.github/workflows/ci.yml). Диаграмма pipeline (jobs, артефакты, `needs`, события, секреты): [docs/ci-cd.md](docs/ci-cd.md). Публикация в registries: [docs/packaging.md](docs/packaging.md).

## License

MIT — see [LICENSE](LICENSE).
