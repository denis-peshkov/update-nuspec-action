#!/bin/sh
set -e

WORKSPACE_ROOT="/github/workspace"
DIR="${1:-.}"

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

exec /UpdateNuspecTool "$DIR"
