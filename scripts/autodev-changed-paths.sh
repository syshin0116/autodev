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

records=$(tr -cd '\0' < "$raw" | wc -c | tr -d ' ')
paths=$(tr '\0' '\n' < "$raw" | sed '/^$/d')
[ -z "$paths" ] && { [ "$records" = "0" ] || { echo "a changed path is empty" >&2; exit 1; }; exit 0; }

# A newline inside a name would split one path into two lines, and the second
# half would be checked against the protected paths instead of the real name.
[ "$records" = "$(printf '%s\n' "$paths" | wc -l | tr -d ' ')" ] \
  || { echo "a changed path contains a newline and cannot be checked safely" >&2; exit 1; }

printf '%s\n' "$paths"
