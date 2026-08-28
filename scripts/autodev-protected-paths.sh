#!/usr/bin/env bash
# Turns approved protected path globs on stdin into one extended regular
# expression that matches a repository-relative path.
#
# The approved policy is the only source of these paths, so the runner never
# carries its own copy of the list.
#
# Usage: printf '%s\n' '.autodev/**' | autodev-protected-paths.sh
set -euo pipefail

patterns=$(cat)
[ -n "$patterns" ] || { echo "no protected paths were supplied" >&2; exit 1; }

# The conversion below parks globstars on a sentinel, so a pattern that
# contains the sentinel itself would be rewritten into something it is not.
if printf '%s' "$patterns" | grep -q '@@GLOBSTAR'; then
  echo "a protected path may not contain @@GLOBSTAR" >&2
  exit 1
fi

# `**/` matches zero or more directories in either position, so both
# **/README.md and docs/**/README.md protect a file at the top of their tree.
expression=$(printf '%s\n' "$patterns" | sed \
  -e 's/[].[^$()+?{}|\\]/\\&/g' \
  -e 's|^\*\*/|@@GLOBSTARROOT@@|' \
  -e 's|/\*\*/|/@@GLOBSTARDIR@@|g' \
  -e 's/\*\*/@@GLOBSTAR@@/g' \
  -e 's/\*/[^\/]*/g' \
  -e 's|@@GLOBSTARROOT@@|(.*/)?|' \
  -e 's|@@GLOBSTARDIR@@|(.*/)?|g' \
  -e 's/@@GLOBSTAR@@/.*/g' \
  | paste -sd'|' -)

printf '^(%s)$\n' "$expression"
