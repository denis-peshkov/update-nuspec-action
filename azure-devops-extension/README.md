# Azure DevOps extension (UpdateNuspecTool)

Сборка и публикация — в [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (отдельные шаги после Docker smoke tests).

## Release vs preview

| Ветка | GitVersion | VSIX manifest | Extension ID | Marketplace |
|-------|------------|---------------|----------------|-------------|
| `master` | `1.2.3` | `vss-extension.json` | `update-nuspec` | **public** (release) |
| `release/*`, `hotfix/*` | `1.3.0-preview.2` | `vss-extension.preview.json` | `update-nuspec-preview` | **private** + `--share-with` (preview) |

Версия с суффиксом `-preview` задаётся в [`GitVersion.yml`](../GitVersion.yml) (`tag: preview` на ветках release/hotfix).

## Шаги CI (ADO)

1. Publish ADO tool (linux-x64 / win-x64) — `semVer` из GitVersion  
2. Build ADO task wrapper (`npm ci`, `tsc`)  
3. Install TFX CLI  
4. Update ADO task.json version  
5. Build ADO extension manifest → `.vss-extension.build.json`  
6. Create ADO extension VSIX  
7. Restore ADO task.json (`git checkout`)  
8. Upload artifact / Publish (release или preview)

## Секреты и переменные GitHub

| Secret / Variable | Назначение |
|-------------------|------------|
| `secrets.AZDO_MARKETPLACE_PAT` | PAT: **Marketplace (Publish)** |
| `vars.ADO_PUBLISH_EXTENSION` | `true` — публиковать в Marketplace после сборки VSIX |
| `vars.ADO_PUBLISHER_ID` | Publisher (по умолчанию `denis-peshkov`) |
| `vars.ADO_TEST_ORGANIZATION` | Org для preview (`--share-with`) |

## Задача в pipeline

```yaml
- task: UseDotNet@2
  inputs:
    packageType: runtime
    version: 8.0.x

- task: UpdateNuspec@1
  inputs:
    dir: '$(Build.SourcesDirectory)'
    dryRun: false
```

Стабильная установка: extension **update-nuspec**. Preview: **update-nuspec-preview** (только shared org).
