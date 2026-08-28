#!/usr/bin/env bash
# Lists every repository-relative path an episode changed, one per line.
#
# Both sides of a rename are listed, because moving a file out of a protected
# directory is a change to that directory. Paths arrive NUL-delimited so a
# space in a name cannot split one path into two.
#
# Usage: autodev-changed-paths.sh <worktree>
set -euo pipefail

tree=${1:?worktree path}
raw=$(mktemp)
trap 'rm -f "$raw"' EXIT

{
  git -C "$tree" diff --no-renames --name-only -z HEAD
  git -C "$tree" ls-files -z --others --exclude-standard
} > "$raw"

# Git separates these paths with NUL and never with a newline, so any newline
# byte here came from inside a name. Such a name cannot be listed one per line,
# and a truncated half would be checked against the protected paths instead of
# the real name.
if [ "$(tr -cd '\n' < "$raw" | wc -c | tr -d ' ')" != "0" ]; then
  echo "a changed path contains a newline and cannot be checked safely" >&2
  exit 1
fi

tr '\0' '\n' < "$raw" | sed '$ { /^$/d; }'
