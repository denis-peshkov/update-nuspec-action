#!/bin/sh
set -e

WORKSPACE_ROOT="/github/workspace"
DIR="${1:-.}"
DRY_RUN="${2:-false}"

# Repository root
case "$DIR" in
  ""|.|./)
    DIR="$WORKSPACE_ROOT"
    ;;
  "$WORKSPACE_ROOT"|"$WORKSPACE_ROOT"/*)
    ;;
  /*)
    ;;
  ./*)
    DIR="$WORKSPACE_ROOT/${DIR#./}"
    ;;
  *)
    DIR="$WORKSPACE_ROOT/$DIR"
    ;;
esac

DRY_RUN_ARG=""
case "$DRY_RUN" in
  true|True|TRUE|1|yes|Yes|YES)
    DRY_RUN_ARG="--dry-run"
    ;;
esac

/UpdateNuspecTool $DRY_RUN_ARG "$DIR"

if [ -n "$PACKAGE_VERSION" ] && [ -n "$GITHUB_OUTPUT" ]; then
  echo "packageVersion=$PACKAGE_VERSION" >> "$GITHUB_OUTPUT"
fi
