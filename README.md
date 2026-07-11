# Homebrew preview tap

Preview builds of [update-nuspec](https://github.com/denis-peshkov/update-nuspec-action) for `release/*` and `hotfix/*` branches.

This branch is updated by CI. Do not commit here manually.

## Install

```bash
brew tap denis-peshkov/update-nuspec https://github.com/denis-peshkov/update-nuspec-action --branch homebrew-preview-tap
brew install update-nuspec-preview
```

Stable releases: `brew install update-nuspec` (homebrew-core, after formula merge).
