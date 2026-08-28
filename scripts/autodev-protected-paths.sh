#!/usr/bin/env bash
# Turns approved protected path globs on stdin into one extended regular
# expression that matches a repository-relative path.
#
# The approved policy is the only source of these paths, so the runner never
# carries its own copy of the list.
#
# Usage: printf '%s\n' '.autodev/**' | autodev-protected-paths.sh
set -euo pipefail

# `**/` matches zero or more directories, so docs/**/README.md must also
# protect docs/README.md.
expression=$(sed \
  -e 's/[].[^$()+?{}|\\]/\\&/g' \
  -e 's|/\*\*/|/@@GLOBSTARDIR@@|g' \
  -e 's/\*\*/@@GLOBSTAR@@/g' \
  -e 's/\*/[^\/]*/g' \
  -e 's|@@GLOBSTARDIR@@|(.*/)?|g' \
  -e 's/@@GLOBSTAR@@/.*/g' \
  | paste -sd'|' -)

[ -n "$expression" ] || { echo "no protected paths were supplied" >&2; exit 1; }
printf '^(%s)$\n' "$expression"
