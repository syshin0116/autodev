#!/usr/bin/env bash
# Reads the durable authorization record for one issue.
#
# The trusted controller and the local runner both call this, so the record
# format has one reader. Prints {"comment_id": <id|null>, "authorizations": []}
# with an empty list when the issue has no record yet.
#
# Usage: autodev-episode-record.sh <owner/repo> <issue-number>
# Tests set AUTODEV_COMMENTS_FILE to a captured comments response instead.
set -euo pipefail

repository=${1:?owner/repo}
issue=${2:?issue number}

export STATE_MARKER='<!-- autodev:authorization -->'

# --slurp cannot be combined with --jq, so the filter runs in a separate jq.
if [ -n "${AUTODEV_COMMENTS_FILE:-}" ]; then
  cat "$AUTODEV_COMMENTS_FILE"
else
  gh api "repos/$repository/issues/$issue/comments" --paginate --slurp
fi > /tmp/autodev-comments.$$.json

trap 'rm -f /tmp/autodev-comments.$$.json' EXIT

comment=$(jq '[.[][] | select(.body | startswith(env.STATE_MARKER))] | last // {}' \
  /tmp/autodev-comments.$$.json)

comment_id=$(jq -r '.id // ""' <<< "$comment")
if [ -z "$comment_id" ]; then
  echo '{"comment_id": null, "authorizations": []}'
  exit 0
fi

authorizations=$(jq -r '.body' <<< "$comment" | sed -n '/^```json$/,/^```$/p' | sed '1d;$d')
[ -n "$authorizations" ] || { echo "authorization record comment $comment_id has no JSON block" >&2; exit 1; }
jq -e 'type == "array"' <<< "$authorizations" > /dev/null \
  || { echo "authorization record comment $comment_id is not an array" >&2; exit 1; }

jq -n --argjson comment_id "$comment_id" --argjson authorizations "$authorizations" \
  '{comment_id: $comment_id, authorizations: $authorizations}'
