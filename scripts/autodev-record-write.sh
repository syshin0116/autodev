#!/usr/bin/env bash
# Writes the durable authorization record for one issue.
#
# Pairs with autodev-episode-record.sh: that script is the only reader, this is
# the only writer, so the comment format has one definition on each side.
#
# Usage: autodev-record-write.sh <owner/repo> <issue> <comment-id|""> <authorizations.json>
# Set AUTODEV_RECORD_DRY_RUN=1 to print the comment body instead of writing it.
set -euo pipefail

repository=${1:?owner/repo}
issue=${2:?issue number}
comment_id=${3-}
authorizations=${4:?authorizations json path}

jq -e 'type == "array" and length > 0' "$authorizations" > /dev/null \
  || { echo "authorizations must be a non-empty array" >&2; exit 1; }

body=$(mktemp)
trap 'rm -f "$body"' EXIT
{
  printf '%s\n\n' '<!-- autodev:authorization -->'
  printf 'Autodev authorization record, written by the trusted controller.\n\n'
  printf '```json\n'
  jq '.' "$authorizations"
  printf '```\n'
} > "$body"

if [ -n "${AUTODEV_RECORD_DRY_RUN:-}" ]; then
  cat "$body"
  exit 0
fi

if [ -n "$comment_id" ]; then
  gh api -X PATCH "repos/$repository/issues/comments/$comment_id" -F body=@"$body" > /dev/null
else
  gh api -X POST "repos/$repository/issues/$issue/comments" -F body=@"$body" > /dev/null
fi
