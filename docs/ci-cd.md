# CI/CD — детальная диаграмма

Оркестратор: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

Триггеры: **push** (`master`, `release/*`, `hotfix/*`), **pull_request**, **workflow_dispatch**.

---

## Mermaid

```mermaid
flowchart TD
  subgraph triggers["Триггеры"]
    T1["push: master, release/*, hotfix/*"]
    T2["pull_request"]
    T3["workflow_dispatch"]
  end

  V["version<br/>GitVersion"]

  subgraph matrix["release-binaries (matrix × 4)"]
    M1["linux-musl · ubuntu"]
    M2["aarch64-macos · macos"]
    M3["x86_64-macos · macos"]
    M4["windows-msvc · windows"]
  end

  TEST["test<br/>Rust + .NET + Sonar"]
  PT["push-tags<br/>master only"]

  PGA["publish-github-action<br/>GHCR + Docker smoke"]
  PADO["publish-ado-extension<br/>VSIX + ADO Marketplace"]

  subgraph pkg["Package managers"]
    CHR["publish-chocolatey"]
    HB["publish-homebrew<br/>master only"]
    HBT["publish-homebrew-tap<br/>release/hotfix"]
  end

  REL["publish-release<br/>master only"]

  triggers --> V
  V --> matrix
  matrix --> TEST
  TEST --> PT
  PT --> PGA
  PT --> PADO
  PT --> CHR
  PT --> HB
  TEST --> HBT
  PADO --> REL
  PT --> REL
```

---

## ASCII (полная цепочка)

```
ci.yml
│
├─ version                              [ubuntu-latest]
│     └─ GitVersion → version, major, minor, channel, prerelease
│
├─ release-binaries (matrix × 4)        [needs: version, fail-fast: false]
│     ├─ binary-x86_64-unknown-linux-musl       ubuntu-latest
│     ├─ binary-aarch64-apple-darwin            macos-latest
│     ├─ binary-x86_64-apple-darwin             macos-latest
│     └─ binary-x86_64-pc-windows-msvc          windows-latest
│           │
│           ├─ всегда:              ado-binary-{target}     (linux + windows)
│           └─ на push:              release-binary-{target} (все 4)
│
├─ test                                 [needs: version, release-binaries]
│     ├─ Rust tests + lcov
│     ├─ .NET tests + OpenCover
│     └─ SonarCloud
│
├─ push-tags                            [needs: test]  if: push + master
│     └─ git tags: v{version}, v{X.Y}, v{X}
│
├─ ═════════ параллельно (после test; на master — после push-tags) ═════════
│
├─ publish-github-action                «GHCR + Docker smoke»
│     ├─ download ado-binary-*
│     ├─ docker build
│     ├─ smoke × 3
│     └─ GHCR push (if push):
│           master:         :version, :X.Y, :X, :latest
│           release/hotfix:  :version only
│
├─ publish-ado-extension                «VSIX + ADO Marketplace»
│     ├─ download ado-binary-*
│     ├─ yarn build task
│     ├─ tfx → VSIX → artifact ado-extension-vsix
│     └─ ADO Marketplace (if push + master)
│
├─ publish-chocolatey                   if: push (master/release/hotfix)
│     ├─ download release-binary-*
│     └─ .nupkg → chocolatey.org
│
└─ publish-release               [needs: push-tags, publish-ado-extension]
      ├─ package update-nuspec-{version}-src.tar.gz
      ├─ download ado-extension-vsix
      ├─ download release-binary-*
      ├─ SHA256SUMS (binaries + src)
      └─ GitHub Release v{version}

publish-homebrew                        if: push + master [needs: publish-release]
      ├─ download src archive from Release → sha256
      ├─ brew install / test / audit
      └─ brew bump-formula-pr / fork PR
```

### Composite actions → шаги

| Job (UI name) | Action | Что делает |
|---------------|--------|------------|
| `version` | [`version`](../.github/actions/version/action.yml) | GitVersion |
| `binary-*` | [`release-binary`](../.github/actions/release-binary/action.yml) | `cargo build --release` |
| `test` | [`test`](../.github/actions/test/action.yml) | Rust + .NET + Sonar |
| `push-tags` | [`push-tags`](../.github/actions/push-tags/action.yml) | Push git tags `v{version}`, `v{X.Y}`, `v{X}` |
| `publish-github-action` (**GHCR + Docker smoke**) | [`publish-github-action`](../.github/actions/publish-github-action/action.yml) | Docker smoke + GHCR |
| `publish-ado-extension` (**VSIX + ADO Marketplace**) | [`publish-ado-extension`](../.github/actions/publish-ado-extension/action.yml) | VSIX + Marketplace |
| `publish-chocolatey` | [`publish-chocolatey`](../.github/actions/publish-chocolatey/action.yml) | Chocolatey pack/push |
| `publish-homebrew` | [`publish-homebrew`](../.github/actions/publish-homebrew/action.yml) | Homebrew core formula |
| `publish-homebrew-tap` | [`publish-homebrew-tap`](../.github/actions/publish-homebrew-tap/action.yml) | Preview tap branch `homebrew-preview-tap` |
| `publish-release` | [`publish-release`](../.github/actions/publish-release/action.yml) | GitHub Release assets |

---

## Артефакты

```
release-binaries (matrix)
  ├─ ado-binary-linux-musl      ──┬─→ publish-github-action (Docker)
  │                               └─→ publish-ado-extension (task binaries)
  ├─ ado-binary-windows-msvc    ────→ publish-ado-extension
  └─ release-binary-*           ──┬─→ publish-chocolatey
                                  └─→ publish-release

publish-ado-extension
  └─ ado-extension-vsix         ────→ publish-release
```

| Артефакт | Создаёт | Потребляет |
|----------|---------|------------|
| `ado-binary-{target}` | `release-binary` (linux + windows) | `publish-github-action`, `publish-ado-extension` |
| `release-binary-{target}` | `release-binary` (на push) | `publish-chocolatey`, `publish-release` |
| `ado-extension-vsix` | `publish-ado-extension` | `publish-release` |

---

## Зависимости (`needs`)

| Job | Зависит от | Условие запуска |
|-----|------------|----------------|
| `version` | — | всегда |
| `release-binaries` | `version`, `test` | всегда |
| `test` | `version` | всегда |
| `push-tags` | `version`, `release-binaries` | `push` + `master` |
| `publish-github-action` | `version`, `release-binaries` | `release-binaries` success |
| `publish-ado-extension` | `version`, `release-binaries` | `release-binaries` success |
| `publish-chocolatey` | `version`, `release-binaries` | `release-binaries` success + `push` на `master` / `release/*` / `hotfix/*` |
| `publish-homebrew` | `version`, `publish-release` | `push` + `master` |
| `publish-homebrew-tap` | `version`, `test` | `push` + `release/*` / `hotfix/*` |
| `publish-release` | `version`, `release-binaries`, `push-tags`, `publish-ado-extension` | `push` + `master`, all needs success |

### Критический путь (`master` push)

```
matrix (4 OS, самый долгий)
  → test
    → release-binaries
         ├─ push-tags (master only)
         ├─ параллельно: publish-github-action | publish-ado-extension | chocolatey
         └─ publish-ado-extension
               → publish-release (release-binaries + push-tags + ado)
                 → publish-homebrew
```

`publish-release` **не ждёт** `publish-github-action`.

---

## По событиям

### `pull_request` / `workflow_dispatch`

```
version → matrix → test
                    ├─ publish-github-action  (только smoke)
                    └─ publish-ado-extension  (только VSIX artifact)
```

`push-tags` и все `publish-*` (кроме пропущенных publish-jobs) — **skipped**.

| Job | GHCR | ADO Marketplace | VSIX artifact | Publish |
|-----|------|-----------------|---------------|---------|
| `publish-github-action` | ❌ (только smoke) | — | — | — |
| `publish-ado-extension` | — | ❌ | ✅ | — |

### `push` → `master`

```
version → matrix → test → release-binaries
                              ├─ push-tags (master)
                              ├─ publish-github-action      (GHCR)
                              ├─ publish-ado-extension        (VSIX + ADO)
                              ├─ publish-chocolatey
                              └─ publish-ado-extension
                                    └─ publish-release
                                          └─ publish-homebrew
```

### `push` → `release/*`, `hotfix/*`

```
version → matrix → test
                    ├─ publish-github-action   (GHCR :version)
                    ├─ publish-ado-extension   (VSIX artifact)
                    ├─ publish-chocolatey
                    └─ publish-homebrew-tap    (branch homebrew-preview-tap)
```

`push-tags`, `publish-homebrew`, `publish-release` — **skipped**.

| Job | GHCR | ADO | Chocolatey | Homebrew preview tap |
|-----|------|-----|------------|----------------------|
| `publish-github-action` | `:version` only | — | — | — |
| `publish-ado-extension` | — | ❌ | — | — |
| `publish-chocolatey` | — | — | ✅ | — |
| `publish-homebrew-tap` | — | — | — | ✅ (`homebrew-preview-tap`) |

---

## Секреты

| Secret | Job / Action | Назначение |
|--------|--------------|------------|
| `SONAR_TOKEN` | `test` | SonarCloud scan |
| `TAGTOKEN` | `push-tags`, `publish-homebrew`, `publish-homebrew-tap` | Git tags; Homebrew core fork push; push branch `homebrew-preview-tap` (`repo` scope) |
| `AZDO_MARKETPLACE_PAT` | `publish-ado-extension` | ADO Marketplace publish (`master`) |
| `CHOCOLATEY_API_KEY` | `publish-chocolatey` | Push `.nupkg` → chocolatey.org |
| `HOMEBREW_GITHUB_API_KEY` | `publish-homebrew` | Classic PAT with **`public_repo`** for `gh pr create`, REST PR API, and `brew bump-formula-pr`; fallback to `TAGTOKEN` on initial PR |
| `GITHUB_TOKEN` | `publish-github-action` | GHCR login + push |

Все секреты передаются в composite actions через `with:` в workflow (не через `secrets:` на уровне шага composite action).

Публикация в Homebrew / Chocolatey / GitHub Release assets: [distribution.md](distribution.md).
