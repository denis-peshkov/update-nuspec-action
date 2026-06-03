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

Build and run locally (amd64):

```bash
docker build -t update-nuspec-action:local .
docker run --rm -v "$PWD:/github/workspace" update-nuspec-action:local .github/fixtures/sample
```

CI runs `docker build` and a smoke test on push/PR (see `.github/workflows/ci.yml`).

Example fixture for manual runs: `.github/fixtures/sample/` (`.csproj` + `.nuspec`).

## License

MIT — see [LICENSE](LICENSE).
