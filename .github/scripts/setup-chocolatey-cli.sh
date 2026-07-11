#!/usr/bin/env bash
set -euo pipefail

if command -v choco >/dev/null 2>&1 && choco --version >/dev/null 2>&1; then
  echo "choco already available: $(choco --version)"
  exit 0
fi

if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
  echo "Chocolatey CLI is expected on Windows runners; choco not found on PATH." >&2
  exit 1
fi

echo "Installing Chocolatey CLI (mono) for choco pack…"
sudo apt-get update -qq
sudo apt-get install -y -qq mono-complete

curl -fsSL https://community.chocolatey.org/install.sh | sudo bash

if [[ -x /opt/chocolatey/bin/choco ]]; then
  echo "/opt/chocolatey/bin" >> "${GITHUB_PATH:-/dev/null}"
  export PATH="/opt/chocolatey/bin:${PATH}"
fi

if ! command -v choco >/dev/null 2>&1 || ! choco --version >/dev/null 2>&1; then
  echo "Chocolatey CLI installation failed; choco is not available." >&2
  exit 1
fi

echo "choco installed: $(choco --version)"
