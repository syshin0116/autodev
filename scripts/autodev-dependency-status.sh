#!/usr/bin/env bash
# Builds one dependency status per blocking issue of a task snapshot.
#
# A dependency counts as complete only when its own authorization record
# reached the merged terminal state, which is what the merge controller writes.
# A dependency with no record still gets an entry, because the readiness check
# requires one status per declared dependency.
#
# Usage: autodev-dependency-status.sh <owner/repo> <snapshot.json> <base-branch>
set -euo pipefail

repository=${1:?owner/repo}
snapshot=${2:?snapshot path}
base=${3:?base branch}

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
: > "$work/entries.jsonl"

while read -r number node_id; do
  [ -n "$number" ] || continue
  record=$("$root/scripts/autodev-episode-record.sh" "$repository" "$number")
  jq -n -c \
    --arg node_id "$node_id" \
    --arg base "$base" \
    --argjson record "$record" \
    '($record.authorizations | max_by(.episode.authorization_generation)) as $episode
     | ($episode != null and $episode.status == "merged") as $merged
     | {
         issue_node_id: $node_id,
         approved_task_sha256: (if $episode == null then null else $episode.episode.task_sha256 end),
         project_revision_sha256: ($episode.project_revision_sha256 // ""),
         authorization_generation: ($episode.episode.authorization_generation // 0),
         evidence_verified: $merged,
         evidence_task_sha256: (if $merged then $episode.episode.task_sha256 else "" end),
         evidence_project_revision_sha256: (if $merged then $episode.project_revision_sha256 else "" end),
         evidence_authorization_generation: (if $merged then $episode.episode.authorization_generation else 0 end),
         merged_into: (if $merged then $base else null end)
       }' >> "$work/entries.jsonl"
done < <(jq -r '.task_snapshot.projection.blocked_by[] | "\(.issue_number) \(.issue_node_id)"' "$snapshot")

jq -s '.' "$work/entries.jsonl"
