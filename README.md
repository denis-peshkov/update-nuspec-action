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

GitHub Action (Docker) that scans .NET projects in a directory and updates the `<dependencies>` section in matching `*.nuspec` files according to `PackageReference` versions from `.csproj`.

## Usage

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

- Recursively looks for `*.nuspec` under `dir`.
- Reads package versions from related `.csproj` (`PackageReference`).
- Rewrites the `<dependencies>` node in each `.nuspec`.
- Exits with code `0` if no `.nuspec` files are found (prints `*.nuspec files not found!`).
- Prints an error and exits non-zero if `dir` does not exist.

## Requirements

- **Runner:** `ubuntu-latest` (or another **linux/amd64** host). The bundled tool is published for `linux-x64`.
- **.NET:** The image includes .NET 8 runtime (framework-dependent apphost).

## Versioning (this repository)

On push to `master` / `release/*` / `hotfix/*`, CI runs [GitVersion](https://gitversion.net/) and exports **`env.semVer`**, then creates tag **`v${{ env.semVer }}`** (for example `v1.0.1`).

`GitVersion.yml` sets `next-version: 1.0.0`. After the first tagged release, version increments follow GitVersion rules and commit history.

To publish a new action version after CI pushed tag `vX.Y.Z`:

1. Open [Releases](https://github.com/denis-peshkov/update-nuspec-action/releases) and create a release for the new tag (or use `gh release create vX.Y.Z`).
2. Update **`vars.semVer`** (and `env.semVer` in your consumer workflows) to `vX.Y.Z`.

## Development

The action image builds **`UpdateNuspecTool`** from source (`UpdateNuspecTool/`) for `linux-x64` during `docker build` (multi-stage `Dockerfile`). The old binary in `tools/` is not used.

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

CLI options: `--help` / `-h`, `--version` / `-v`, `--dry-run` / `-d` / `--demo` (or positional `true`).

```bash
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- --help
dotnet run --project UpdateNuspecTool/UpdateNuspecTool.csproj -- --version
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

Build and run the action image (Linux x64 only):

```bash
docker build --platform linux/amd64 -t update-nuspec-action:local .
docker run --rm --platform linux/amd64 \
  -v "$PWD:/github/workspace" \
  update-nuspec-action:local UpdateNuspecTool.Tests/TestData/
```

CI runs `dotnet test`, `docker build`, and smoke tests on push/PR (see `.github/workflows/ci.yml`).

```bash
dotnet test UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj
```

Test fixtures: `UpdateNuspecTool.Tests/TestData/` (`config.nuspec`, `cgf.nuspec`, `Cross.Messaging.nuspec`, …).

## License

MIT — see [LICENSE](LICENSE).
