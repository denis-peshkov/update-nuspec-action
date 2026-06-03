[![License](https://img.shields.io/github/license/denis-peshkov/update-nuspec-action)](LICENSE)
[![GitHub Release Date](https://img.shields.io/github/release-date/denis-peshkov/update-nuspec-action?label=released)](https://github.com/denis-peshkov/update-nuspec-action/releases)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=update-nuspec-action&metric=coverage)](https://sonarcloud.io/summary/new_code?id=update-nuspec-action)
[![issues](https://img.shields.io/github/issues/denis-peshkov/update-nuspec-action)](https://github.com/denis-peshkov/update-nuspec-action/issues)
[![CI](https://github.com/denis-peshkov/update-nuspec-action/actions/workflows/ci.yml/badge.svg?event=pull_request)](https://github.com/denis-peshkov/update-nuspec-action/actions/workflows/ci.yml)

![Size](https://img.shields.io/github/repo-size/denis-peshkov/update-nuspec-action)
[![GitHub contributors](https://img.shields.io/github/contributors/denis-peshkov/update-nuspec-action)](https://github.com/denis-peshkov/update-nuspec-action/contributors)
[![GitHub commits since latest release (by date)](https://img.shields.io/github/commits-since/denis-peshkov/update-nuspec-action/latest?label=new+commits)](https://github.com/denis-peshkov/update-nuspec-action/commits/master)
![Activity](https://img.shields.io/github/commit-activity/w/denis-peshkov/update-nuspec-action)
![Activity](https://img.shields.io/github/commit-activity/m/denis-peshkov/update-nuspec-action)
![Activity](https://img.shields.io/github/commit-activity/y/denis-peshkov/update-nuspec-action)

# update-nuspec-action

GitHub Action (Docker) that scans .NET projects in a directory and updates the `<dependencies>` section in matching `*.nuspec` files according to `PackageReference` versions from the related `.csproj` (project name = `<id>` in nuspec metadata).

## Usage

Pin a [release tag](https://github.com/denis-peshkov/update-nuspec-action/releases) (recommended):

```yaml
- uses: denis-peshkov/update-nuspec-action@v1
```

With a custom scan directory:

```yaml
- uses: denis-peshkov/update-nuspec-action@v1
  with:
    dir: src/MyPackage
```

Equivalent to `/github/workspace/src/MyPackage` inside the container. An absolute path (starting with `/`) is used as-is.

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `dir` | No | `.` | Folder with `.csproj` and `.nuspec`, relative to `/github/workspace` (`.` = repo root) |

## Behavior

- Looks for `*.nuspec` in `dir` (top level only, not subfolders).
- Loads `{id}.csproj` from the same `dir`, where `{id}` is `<metadata><id>`.
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
- **Dry-run** (`--dry-run`, `-d`, `--demo`, or positional `true`): full report, no file save (`[DRY RUN]` in the log).

Example multi-TFM project: `UpdateNuspecTool.Tests/TestData/Cross.Messaging.csproj` + `Cross.Messaging.nuspec`.

## Requirements

- **Runner:** `ubuntu-latest` (or another **linux/amd64** host). The bundled tool is published for `linux-x64`.
- **.NET:** The image includes .NET 8 runtime (framework-dependent apphost).

## Versioning (this repository)

On push to `master` / `release/*` / `hotfix/*`, CI runs [GitVersion](https://gitversion.net/) and exports **`env.semVer`**, then creates tag **`v${{ env.semVer }}`** (for example `v0.2.1`).

1. Checks out with `fetch-depth: 0` (full history for [GitVersion](https://gitversion.net/)).
2. Runs `dotnet test`.
3. Builds the Docker image with GitVersion MSBuild properties (`Version`, `AssemblyVersion`, `FileVersion`, `InformationalVersion`, assembly metadata).
4. Smoke-tests the image.
5. On protected branches, creates and pushes tag **`v${{ env.semVer }}`**.

CI (`.github/workflows/ci.yml`) also runs `dotnet test`, builds the Docker image with GitVersion MSBuild properties (`Version`, `AssemblyVersion`, `FileVersion`, `InformationalVersion`, …), and smoke-tests the image. Checkout uses `fetch-depth: 0` for GitVersion only.

To publish a new action version after CI pushed tag `vX.Y.Z`:

1. Open [Releases](https://github.com/denis-peshkov/update-nuspec-action/releases) and create a release for the new tag (or use `gh release create vX.Y.Z`).
2. In other repositories, set `uses: denis-peshkov/update-nuspec-action@vX.Y.Z` to that tag.

## Development

### Repository layout

| Path | Role |
|------|------|
| `UpdateNuspecTool/` | CLI source |
| `UpdateNuspecTool.Tests/` | NUnit tests and fixtures |
| `UpdateNuspecTool.Tests/TestData/` | Sample `.nuspec` / `.csproj` pairs |
| `Dockerfile` | Multi-stage image: SDK build → runtime + `entrypoint.sh` |
| `action.yml` | Action metadata; runs the Docker image |

### Tests

```bash
dotnet test UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj --configuration Release
```

Fixtures: `UpdateNuspecTool.Tests/TestData/` (`MyPackage.nuspec`, `Cross.Messaging.nuspec`, `config.nuspec`, `cgf.nuspec`, …).

### CLI (local)

Options: `--help` / `-h`, `--version` / `-v`, `--dry-run` / `-d` / `--demo` (or positional `true`).

```bash
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- --help
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- --version
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- UpdateNuspecTool.Tests/TestData --dry-run
```

Publish the tool locally (same flags; change `-r` and output folder per OS/CPU):

**Linux (x64)** — used in the action Docker image and `ubuntu-latest`:

```bash
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
  -c Release \
  -r linux-x64 \
  --self-contained false \
  -p:PublishSingleFile=true \
  -o ./artifacts/publish/linux-x64

./artifacts/publish/linux-x64/UpdateNuspecTool ./UpdateNuspecTool.Tests/TestData

# Demo / test run: full report in console, no file changes
./artifacts/publish/linux-x64/UpdateNuspecTool ./UpdateNuspecTool.Tests/TestData --dry-run
```

**Windows (x64):**

```powershell
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj `
  -c Release `
  -r win-x64 `
  --self-contained false `
  -p:PublishSingleFile=true `
  -o ./artifacts/publish/win-x64

./artifacts/publish/win-x64/UpdateNuspecTool.exe ./UpdateNuspecTool.Tests/TestData
```

**macOS (Apple Silicon, ARM64):**

```bash
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
  -c Release \
  -r osx-arm64 \
  --self-contained false \
  -p:PublishSingleFile=true \
  -o ./artifacts/publish/osx-arm64

./artifacts/publish/osx-arm64/UpdateNuspecTool ./UpdateNuspecTool.Tests/TestData
```

| Platform | Runtime ID (`-r`) | Output executable |
|----------|-------------------|-------------------|
| Linux x64 | `linux-x64` | `UpdateNuspecTool` |
| Windows x64 | `win-x64` | `UpdateNuspecTool.exe` |
| macOS ARM64 | `osx-arm64` | `UpdateNuspecTool` |

Other common RIDs: `linux-arm64`, `win-arm64`, `osx-x64`.

### Docker image

Build and run the action image (Linux x64 only):

```bash
docker build --platform linux/amd64 -t update-nuspec-action:local .
docker run --rm --platform linux/amd64 \
  -v "$PWD:/github/workspace" \
  update-nuspec-action:local UpdateNuspecTool.Tests/TestData/
```

CI passes GitVersion values as build arguments (same idea as `dotnet build` with `-p:Version=...`):

```bash
docker build --platform linux/amd64 -t update-nuspec-action:local . \
  --build-arg VERSION="0.2.0" \
  --build-arg ASSEMBLY_VERSION="0.2.0.0" \
  --build-arg FILE_VERSION="0.2.0.0" \
  --build-arg INFORMATIONAL_VERSION="0.2.0+abc123"
```

Optional metadata args: `COMPANY`, `PRODUCT`, `DESCRIPTION`, `REPOSITORY_URL`, `REPOSITORY_TYPE`, `CLS_COMPLIANT`, `NEUTRAL_LANGUAGE`, `BUILD_CONFIG`.

On Apple Silicon hosts, use `--platform linux/amd64` so the image matches GitHub-hosted runners.

CI runs `dotnet test`, `docker build`, and smoke tests on push/PR (see `.github/workflows/ci.yml`).

## License

MIT — see [LICENSE](LICENSE).
