#!/usr/bin/env bash
# Builds the integrity-filtered projection the Agent is allowed to see.
#
# The trusted controller and the local runner both call this, so the recorded
# agent input digest stays comparable. Changing the output changes that digest
# and invalidates in-flight episodes.
#
# Usage: autodev-agent-input.sh <snapshot.json> <author-association>
set -euo pipefail

snapshot=${1:?snapshot path}
association=${2:?issue author association}

case "$association" in
  OWNER | MEMBER | COLLABORATOR) trusted=true ;;
  *) trusted=false ;;
esac

jq -r '"# " + .task_snapshot.projection.title' "$snapshot"
echo
if [ "$trusted" = true ]; then
  jq -r '.task_snapshot.projection.body // ""' "$snapshot"
else
  echo "The issue body was withheld because its author association is $association."
fi
echo
jq -r '"Blocked by: " + ([.task_snapshot.projection.blocked_by[].issue_number | tostring] | join(", ") | if . == "" then "none" else . end)' "$snapshot"
