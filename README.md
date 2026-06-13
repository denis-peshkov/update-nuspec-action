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

GitHub Action (Docker) that scans .NET projects in a directory and updates the `<dependencies>` section in matching `*.nuspec` files according to `PackageReference` versions from the related `.csproj` (project name = `<id>` in nuspec metadata). Optionally updates `package.json` version and scoped npm dependencies.

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
- **.NET:** The image includes .NET 8 runtime (framework-dependent apphost).
- **Colored log output:** enabled by default in the image (`CONSOLE_ANSI_COLOR=true`). Override with `env: CONSOLE_ANSI_COLOR: false` on the step if needed.

**On Windows:** use the CLI (`dotnet publish -r win-x64`, see [CLI (local)](#cli-local)) or the [Azure DevOps extension](#azure-devops-extension) (`UpdateNuspec@1` on `windows-latest`).

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

| Branch | SemVer (example) | Git tag | GitHub Release | ADO extension |
|--------|------------------|---------|----------------|---------------|
| `master` | `1.2.3` (stable) | `v1.2.3`, `v1.2`, `v1` | **Release** (not prerelease) | `update-nuspec` → Marketplace **public** |
| `release/*`, `hotfix/*` | `1.3.0-preview.4` | `v1.3.0-preview.4` | **Pre-release** | `update-nuspec` (version shared via `--share-with peshkov` only) |

On `release/*` and `hotfix/*`, GitVersion already uses `tag: preview` — no extra configuration is required.

CI also creates a [GitHub Release](https://github.com/denis-peshkov/update-nuspec-action/releases) with a `.vsix` artifact (after the tag is pushed).

ADO Marketplace publish uses secret `AZDO_MARKETPLACE_PAT` (scope **Marketplace (Publish)**), publisher **peshkov**.

After merge to `master`, CI updates tags **`v{major}`**, **`v{major}.{minor}`**, and **`v{semVer}`** (for example `v1`, `v1.2`, `v1.2.3`) on the commit that contains the current `Dockerfile` (the tool is built inside the image). Use:

```yaml
uses: denis-peshkov/update-nuspec-action@v1      # latest stable 1.x.y on master
# or an exact release:
uses: denis-peshkov/update-nuspec-action@v1.2.3
```

The `@v1` tag is a **moving** pointer to the latest major-1 release; after breaking changes in 2.x, switch to `@v2`.

## Azure DevOps extension

The same tool is available as pipeline task **`UpdateNuspec@1`** on [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=peshkov.update-nuspec). Usage, examples, inputs, and preview install: [azure-devops-extension/marketplace/overview.md](azure-devops-extension/marketplace/overview.md).

## Development

### Repository layout

| Path | Role |
|------|------|
| `UpdateNuspecTool/` | CLI source |
| `UpdateNuspecTool.Tests/` | NUnit tests and fixtures |
| `UpdateNuspecTool.Tests/TestData/` | Sample `.nuspec` / `.csproj` pairs |
| `Dockerfile` | Multi-stage image (`linux/amd64`): `dotnet publish` in build stage, runtime + `entrypoint.sh` |
| `action.yml` | Action metadata; runs the Docker image |
| `azure-devops-extension/` | Extension root (`vss-extension.json`); VSIX build in `.github/workflows/ci.yml` |
| `azure-devops-extension/marketplace/` | Marketplace content: `overview.md`, `license.md`, icon, screenshots |
| `azure-devops-extension/task/` | Pipeline task `UpdateNuspec@1` (TypeScript wrapper + bundled tool binaries) |

### Tests

```bash
dotnet restore UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj
dotnet test UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj --configuration Release --no-restore
```

CI restores the test project in **Restore dependencies**, then runs tests after build (see `.github/workflows/ci.yml`).

Fixtures: `UpdateNuspecTool.Tests/TestData/` (`MyPackage.nuspec`, `Cross.Messaging.nuspec`, `package.json`, …).

### CLI (local)

Options: `--help` / `-h`, `--version` / `-v`, `--dry-run` / `-d` / `--demo` (or positional `true`), `--package-version` / `-pv`, `--dependency-scope` / `-ds`.

```bash
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- --help
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- --version
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- UpdateNuspecTool.Tests/TestData --dry-run
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- ./client/dist/my-app --package-version 1.2.3 --dependency-scope @guru/
```

Publish the tool locally (same flags; change `-r` and output folder per OS/CPU):

**Linux (x64)** — used in the action Docker image and `ubuntu-latest`:

```bash
dotnet restore UpdateNuspecTool/UpdateNuspecTool.csproj -r linux-x64
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
  -c Release \
  --no-restore \
  -r linux-x64 \
  --self-contained false \
  -o UpdateNuspecTool/bin/publish/linux-x64
```

(`PublishSingleFile` is enabled in the `.csproj` when `-r linux-x64` is set. Output under `bin/` is gitignored.)

```bash
UpdateNuspecTool/bin/publish/linux-x64/UpdateNuspecTool UpdateNuspecTool.Tests/TestData

# Demo / test run: full report in console, no file changes
UpdateNuspecTool/bin/publish/linux-x64/UpdateNuspecTool UpdateNuspecTool.Tests/TestData --dry-run
```

**Windows (x64):**

```powershell
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj `
  -c Release `
  -r win-x64 `
  --self-contained false `
  -p:PublishSingleFile=true `
  -o UpdateNuspecTool/bin/publish/win-x64

UpdateNuspecTool/bin/publish/win-x64/UpdateNuspecTool.exe UpdateNuspecTool.Tests/TestData
```

**macOS (Apple Silicon, ARM64):**

```bash
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
  -c Release \
  -r osx-arm64 \
  --self-contained false \
  -p:PublishSingleFile=true \
  -o UpdateNuspecTool/bin/publish/osx-arm64

UpdateNuspecTool/bin/publish/osx-arm64/UpdateNuspecTool UpdateNuspecTool.Tests/TestData
```

| Platform | Runtime ID (`-r`) | Output executable |
|----------|-------------------|-------------------|
| Linux x64 | `linux-x64` | `UpdateNuspecTool` |
| Windows x64 | `win-x64` | `UpdateNuspecTool.exe` |
| macOS ARM64 | `osx-arm64` | `UpdateNuspecTool` |

Other common RIDs: `linux-arm64`, `win-arm64`, `osx-x64`.

### Docker image

The action image builds and publishes the tool inside `Dockerfile`:

```bash
docker build --platform linux/amd64 -t update-nuspec-action:local .
docker run --rm --platform linux/amd64 \
  -v "$PWD:/github/workspace" \
  update-nuspec-action:local UpdateNuspecTool.Tests/TestData

# dry-run: second argument true (same as action input dryRun)
docker run --rm --platform linux/amd64 \
  -v "$PWD:/github/workspace" \
  update-nuspec-action:local UpdateNuspecTool.Tests/TestData true
```

On Apple Silicon hosts, use `--platform linux/amd64` so the image matches GitHub-hosted runners.

Full pipeline on push/PR: restore → build → tests → SonarCloud → `docker build` and smoke tests (`.github/workflows/ci.yml`).

## License

MIT — see [LICENSE](LICENSE).
