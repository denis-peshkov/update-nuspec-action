# Update *.nuspec

Pipeline task that scans a directory for `*.nuspec` files and updates the `<dependencies>` section from matching `PackageReference` entries in `{id}.csproj`. Optionally updates `package.json` (same behavior as the [update-nuspec-action](https://github.com/denis-peshkov/update-nuspec-action) GitHub Action).

## Usage

```yaml
steps:
  - task: UseDotNet@2
    displayName: Use .NET 8 runtime
    inputs:
      packageType: runtime
      version: 8.0.x

  - task: UpdateNuspec@1
    displayName: Sync nuspec dependencies
    inputs:
      dir: '$(Build.SourcesDirectory)'
      dryRun: false
```

### package.json (built npm package)

```yaml
  - task: UpdateNuspec@1
    displayName: Update package version in built package
    inputs:
      dir: 'client/dist/$(proj)'
      packageVersion: '$(GitVersion_SemVer)'
      dependencyScope: '@guru/'   # optional; empty = version only, skip dependency alignment
```

Sets pipeline variable `PackageVersion` when `packageVersion` is provided.

### Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `dir` | No | `$(Build.SourcesDirectory)` | Root folder to scan recursively for `.nuspec` / `.csproj` pairs and (when `packageVersion` is set) `package.json` |
| `dryRun` | No | `false` | Report only; do not write files |
| `packageVersion` | No | *(empty)* | SemVer to set in `package.json` `version` |
| `dependencyScope` | No | *(empty)* | npm package name prefix to set to `^packageVersion`. Skipped when empty |

## Requirements

- **Agent:** `windows-latest` or `ubuntu-latest` (bundled `win-x64` / `linux-x64` tool).
- **.NET:** .NET 8 **runtime** on the agent (`UseDotNet@2`), framework-dependent publish.

## Links

- [Repository](https://github.com/denis-peshkov/update-nuspec-action)
- [Issues](https://github.com/denis-peshkov/update-nuspec-action/issues)
