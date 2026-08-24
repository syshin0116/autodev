#!/usr/bin/env bash
# Reads and writes one marked state comment on an issue.
#
# Every durable record Autodev keeps on an issue uses this, so the comment
# format has one implementation for both directions. Callers pass their marker
# and the JSON key their payload lives under.
#
# Usage:
#   autodev-comment-record.sh read  <owner/repo> <issue> <marker> <key>
#   autodev-comment-record.sh write <owner/repo> <issue> <comment-id|""> <marker> <payload.json>
#
# Tests substitute the comments response with AUTODEV_COMMENTS_FILE, or with
# AUTODEV_COMMENTS_DIR holding one <issue>.json per issue.
# AUTODEV_RECORD_DRY_RUN prints a rendered comment instead of writing it.
set -euo pipefail

mode=${1:?read or write}
shift

case "$mode" in
  read)
    repository=${1:?owner/repo}
    issue=${2:?issue number}
    export MARKER=${3:?marker}
    key=${4:?payload key}

    if [ -n "${AUTODEV_COMMENTS_FILE:-}" ]; then
      comments=$(cat "$AUTODEV_COMMENTS_FILE")
    elif [ -n "${AUTODEV_COMMENTS_DIR:-}" ]; then
      comments=$(cat "$AUTODEV_COMMENTS_DIR/$issue.json" 2> /dev/null || echo '[[]]')
    else
      # --slurp cannot be combined with --jq, so the filter runs separately.
      comments=$(gh api "repos/$repository/issues/$issue/comments" --paginate --slurp)
    fi

    comment=$(jq '[.[][] | select(.body | startswith(env.MARKER))] | last // {}' <<< "$comments")
    comment_id=$(jq -r '.id // ""' <<< "$comment")
    if [ -z "$comment_id" ]; then
      jq -n --arg key "$key" '{comment_id: null} + {($key): []}'
      exit 0
    fi

    payload=$(jq -r '.body' <<< "$comment" | sed -n '/^```json$/,/^```$/p' | sed '1d;$d')
    [ -n "$payload" ] || { echo "record comment $comment_id has no JSON block" >&2; exit 1; }
    jq -e 'type == "array"' <<< "$payload" > /dev/null \
      || { echo "record comment $comment_id is not an array" >&2; exit 1; }
    jq -n --argjson comment_id "$comment_id" --argjson payload "$payload" --arg key "$key" \
      '{comment_id: $comment_id} + {($key): $payload}'
    ;;

  write)
    repository=${1:?owner/repo}
    issue=${2:?issue number}
    comment_id=${3-}
    marker=${4:?marker}
    payload=${5:?payload json path}

    jq -e 'type == "array" and length > 0' "$payload" > /dev/null \
      || { echo "record payload must be a non-empty array" >&2; exit 1; }

    body=$(mktemp)
    trap 'rm -f "$body"' EXIT
    {
      printf '%s\n\n' "$marker"
      printf 'Autodev state, written by the trusted controller and the delivery runner.\n\n'
      printf '```json\n'
      jq '.' "$payload"
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
    ;;

  *) echo "unsupported mode: $mode" >&2; exit 2 ;;
esac
