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

case "$DRY_RUN" in
  true|True|TRUE|1|yes|Yes|YES)
    exec /UpdateNuspecTool --dry-run "$DIR"
    ;;
  *)
    exec /UpdateNuspecTool "$DIR"
    ;;
esac
