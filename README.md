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
    dir: /github/workspace/src/MyPackage
```

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `dir` | No | `/github/workspace` | Path inside the container to the folder with `.csproj` and `.nuspec` files |

In a typical workflow the repository is checked out into `/github/workspace`, so the default covers the repo root.

## Behavior

- Recursively looks for `*.nuspec` under `dir`.
- Reads package versions from related `.csproj` (`PackageReference`).
- Rewrites the `<dependencies>` node in each `.nuspec`.
- Exits with code `0` if no `.nuspec` files are found (prints `*.nuspec files not found!`).
- Prints an error and exits non-zero if `dir` does not exist.

## Requirements

- **Runner:** `ubuntu-latest` (or another **linux/amd64** host). The bundled tool is published for `linux-x64`.
- **.NET:** The image includes .NET 8 runtime (framework-dependent apphost).

## Development

Build and run locally (amd64):

```bash
docker build -t update-nuspec-action:local .
docker run --rm -v "$PWD/.github/fixtures/sample:/work" update-nuspec-action:local /work
```

CI runs `docker build` and a smoke test on push/PR (see `.github/workflows/ci.yml`).

Example fixture for manual runs: `.github/fixtures/sample/` (`.csproj` + `.nuspec`).

## License

MIT — see [LICENSE](LICENSE).
