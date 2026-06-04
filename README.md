[![License](https://img.shields.io/github/license/denis-peshkov/update-nuspec-action)](LICENSE)
[![GitHub Release Date](https://img.shields.io/github/release-date/denis-peshkov/update-nuspec-action?label=released)](https://github.com/denis-peshkov/update-nuspec-action/releases)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=UpdateNuspecTool&metric=coverage)](https://sonarcloud.io/summary/new_code?id=UpdateNuspecTool)
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
| `dir` | No | `.` | Root folder to scan recursively for `.csproj` / `.nuspec` pairs, relative to `/github/workspace` (`.` = repo root) |

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
- **Dry-run** (`--dry-run`, `-d`, `--demo`, or positional `true`): full report, no file save (`[DRY RUN]` in the log).

Example multi-TFM project: `UpdateNuspecTool.Tests/TestData/Cross.Messaging.csproj` + `Cross.Messaging.nuspec`.

## Requirements

- **Runner:** `ubuntu-latest` (or another **linux/amd64** host). The bundled tool is published for `linux-x64`.
- **.NET:** The image includes .NET 8 runtime (framework-dependent apphost).

## Versioning (this repository)

[GitVersion](https://gitversion.net/) (`GitVersion.yml`) on push:

| Branch | SemVer (пример) | Git tag | GitHub Release | ADO extension |
|--------|-----------------|---------|----------------|---------------|
| `master` | `1.2.3` (stable) | `v1.2.3`, `v1.2`, `v1` | **Release** (не prerelease) | `update-nuspec` → Marketplace **public** |
| `release/*`, `hotfix/*` | `1.3.0-preview.4` | `v1.3.0-preview.4` | **Pre-release** | `update-nuspec-dev` (private, shared) |

На `release/*` и `hotfix/*` в GitVersion уже задан `tag: preview` — отдельно настраивать не нужно.

CI также создаёт [GitHub Release](https://github.com/denis-peshkov/update-nuspec-action/releases) с артефактом `.vsix` (после push тега).

Публикация ADO в Marketplace: secret `AZDO_MARKETPLACE_PAT` (scope **Marketplace (Publish)**), publisher **peshkov**.

Для GitHub Action после merge в `master`:

```yaml
uses: denis-peshkov/update-nuspec-action@v1.2.3
```

## Azure DevOps extension

The same tool is packaged as a **Visual Studio Marketplace** extension in the **same CI** workflow ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)): after tests and Docker smoke tests, the pipeline builds `win-x64`/`linux-x64` binaries, compiles the task wrapper, and produces a `.vsix` artifact (`ado-extension-vsix`).

| Канал | Манифест | Extension ID | Marketplace |
|-------|----------|----------------|-------------|
| `master` | `vss-extension.json` | `update-nuspec` | **public** |
| `release/*`, `hotfix/*` | `vss-extension.preview.json` | `update-nuspec-dev` | **private**, CI `--share-with peshkov` |

Версия VSIX для preview: `major.minor.patch.preReleaseNumber` (например `1.1.0.4`); git-теги остаются `1.1.0-preview.4`.

### Установка private preview (`update-nuspec-dev`)

В публичном поиске Marketplace extension **не виден** — это нормально. Нужен успешный publish из CI (или ручной `tfx publish` с `--share-with <org-slug>`).

1. Убедитесь, что org slug в URL совпадает с `--share-with` в CI (сейчас **peshkov** → `https://dev.azure.com/peshkov`).
2. Откройте организацию → **Organization settings** (⚙️) → **Extensions** → вкладка **Shared** (не Browse Marketplace).
3. Найдите **[Dev] Update \*.nuspec** / `peshkov.update-nuspec-dev` → **Install** → выберите org → **Install**.
4. Альтернатива: [Manage Extensions](https://marketplace.visualstudio.com/manage) (publisher **peshkov**) → extension → **Share/Unshare** → добавить org → в org откройте страницу extension по ссылке **Get it free** (видна только после share).

Прямая ссылка на listing (работает после share, без поиска):  
`https://marketplace.visualstudio.com/items?itemName=peshkov.update-nuspec-dev`

Если в **Shared** пусто: проверьте, что CI publish прошёл и в Manage указано *Shared with* ваша org.

```yaml
- task: UseDotNet@2
  inputs:
    packageType: runtime
    version: 8.0.x

# @1 — последняя установленная 1.x.y; не фиксируйте @1.1.0 после обновления extension
- task: UpdateNuspec@1
  inputs:
    dir: '$(Build.SourcesDirectory)'
    dryRun: false
```

## Development

### Repository layout

| Path | Role |
|------|------|
| `UpdateNuspecTool/` | CLI source |
| `UpdateNuspecTool.Tests/` | NUnit tests and fixtures |
| `UpdateNuspecTool.Tests/TestData/` | Sample `.nuspec` / `.csproj` pairs |
| `Dockerfile` | Runtime image; copies single-file `artifacts/publish/linux-x64/UpdateNuspecTool` from CI **Build** |
| `action.yml` | Action metadata; runs the Docker image |
| `azure-devops-extension/` | Marketplace extension manifest and pipeline task |
| `azure-devops-extension/` | Marketplace extension; сборка VSIX — шаги в `.github/workflows/ci.yml` |

### Tests

```bash
dotnet restore UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj
dotnet test UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj --configuration Release --no-restore
```

CI restores the test project in **Restore dependencies**, then runs tests after the tool is published (see `.github/workflows/ci.yml`).

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
dotnet restore UpdateNuspecTool/UpdateNuspecTool.csproj -r linux-x64
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
  -c Release \
  --no-restore \
  -r linux-x64 \
  --self-contained false \
  -o ./artifacts/publish/linux-x64
```

(`PublishSingleFile` is enabled in the `.csproj` when `-r linux-x64` is set.)

```bash
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

Restore, build, and publish the tool for `linux-x64` (same as the CI **Build** step), then build the image:

```bash
dotnet restore UpdateNuspecTool.Tests/UpdateNuspecTool.Tests.csproj
dotnet restore UpdateNuspecTool/UpdateNuspecTool.csproj -r linux-x64
dotnet build UpdateNuspecTool/UpdateNuspecTool.csproj -c Release --no-restore -r linux-x64
dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj -c Release --no-restore --no-build \
  -r linux-x64 --self-contained false -o artifacts/publish/linux-x64

docker build --platform linux/amd64 -t update-nuspec-action:local .
docker run --rm --platform linux/amd64 \
  -v "$PWD:/github/workspace" \
  update-nuspec-action:local UpdateNuspecTool.Tests/TestData/
```

On Apple Silicon hosts, use `--platform linux/amd64` so the image matches GitHub-hosted runners.

Full pipeline on push/PR: restore → publish tool → tests → SonarCloud → `docker build` and smoke tests (`.github/workflows/ci.yml`).

## License

MIT — see [LICENSE](LICENSE).
