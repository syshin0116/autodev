#!/usr/bin/env bash
# Claims one authorized delivery episode and implements it with the local
# engine. GitHub decided the authorization; this script never decides it.
#
# Usage: scripts/autodev-deliver.sh --issue <number> [--apply]
#
# Without --apply it stops after the checks and prints the writes it would
# perform. The engine receives the integrity-filtered projection and a working
# tree, never a repository credential.
set -euo pipefail

issue=""
apply=false
branch=""
pushed=false
while [ $# -gt 0 ]; do
  case "$1" in
    --issue) issue=${2:?issue number}; shift 2 ;;
    --apply) apply=true; shift ;;
    --help)
      printf '%s\n' \
        "Usage: scripts/autodev-deliver.sh --issue <number> [--apply]" \
        "" \
        "  --issue <number>  Claim the authorized episode for this issue." \
        "  --apply           Required to push a branch or open a pull request." \
        "  --help            Print this usage."
      exit 0
      ;;
    *) echo "unsupported option: $1" >&2; exit 2 ;;
  esac
done
[ -n "$issue" ] || { echo "--issue is required" >&2; exit 2; }

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"

for tool in gh jq cargo codex git; do
  command -v "$tool" > /dev/null || { echo "required tool is missing: $tool" >&2; exit 1; }
done

digest() { shasum -a 256 "$1" | cut -d' ' -f1; }
fail() { echo "STOP: $*" >&2; exit 1; }

repository=$(sed -n '/^task_source:/,/^[a-z]/p' .autodev/config.yaml | sed -n 's/^  repository: //p')
[ -n "$repository" ] || fail "config has no github_issues task source"

work=$(mktemp -d "${TMPDIR:-/tmp}/autodev-episode-XXXXXX")
tree="$work/tree"

# A run that pushes nothing must leave nothing behind, or the next claim for
# the same episode cannot create its branch.
release_episode() {
  if [ -d "$tree" ]; then
    git -C "$tree" diff > "$work/change.diff" 2>/dev/null || true
    git worktree remove --force "$tree" > /dev/null 2>&1 || true
  fi
  if [ "$pushed" != true ] && [ -n "$branch" ]; then
    git branch -D "$branch" > /dev/null 2>&1 || true
  fi
  echo "episode workspace: $work"
}
trap release_episode EXIT

# The durable record is authority for what was authorized. Rebuilding the
# snapshot and the agent input here is what catches a task edited after
# authorization.
scripts/autodev-episode-record.sh "$repository" "$issue" > "$work/record.json"
jq -r '.authorizations' "$work/record.json" > "$work/prior.json"
comment_id=$(jq -r '.comment_id // ""' "$work/record.json")
[ "$(jq -r '.authorizations | length' "$work/record.json")" != "0" ] \
  || fail "issue #$issue has no authorization record; apply autodev:ready first"
jq '.authorizations | max_by(.episode.authorization_generation)' "$work/record.json" > "$work/episode.json"

status=$(jq -r '.status' "$work/episode.json")
[ "$status" = "active" ] || fail "episode status is $status, not active"
generation=$(jq -r '.episode.authorization_generation' "$work/episode.json")

cargo run --locked --quiet -- --print-task-snapshot --root . --issue "$issue" > "$work/snapshot.json"
[ "$(jq -r '.task_snapshot.sha256' "$work/snapshot.json")" = "$(jq -r '.episode.task_sha256' "$work/episode.json")" ] \
  || fail "the task changed after authorization; reauthorize issue #$issue"
[ "$(jq -r '.project_revision.sha256' "$work/snapshot.json")" = "$(jq -r '.project_revision_sha256' "$work/episode.json")" ] \
  || fail "the project revision changed after authorization; reauthorize issue #$issue"

association=$(gh api "repos/$repository/issues/$issue" --jq '.author_association')
scripts/autodev-agent-input.sh "$work/snapshot.json" "$association" > "$work/agent-input.md"
[ "$(digest "$work/agent-input.md")" = "$(jq -r '.agent_input_sha256' "$work/episode.json")" ] \
  || fail "the agent input no longer matches the authorized digest"

base=$(jq -r '.project_revision.projection.delivery.base_branch' "$work/snapshot.json")

# A dependency releases this task only when its own episode reached the merged
# terminal state, so an unfinished predecessor blocks instead of racing.
if [ "$(jq '.task_snapshot.projection.blocked_by | length' "$work/snapshot.json")" != "0" ]; then
  scripts/autodev-dependency-status.sh "$repository" "$work/snapshot.json" "$base" > "$work/dependencies.json"
  ready=$(cargo run --locked --quiet -- --dependencies-ready \
    --root . --issue "$issue" --statuses "$work/dependencies.json" | jq -r '.ready')
  if [ "$ready" != "true" ]; then
    jq -r '.[] | select(.evidence_verified | not) | "incomplete dependency: " + .issue_node_id' \
      "$work/dependencies.json" >&2
    gh issue edit "$issue" --repo "$repository" --add-label "autodev:blocked" > /dev/null
    fail "issue #$issue is blocked by an incomplete dependency"
  fi
  gh issue edit "$issue" --repo "$repository" --remove-label "autodev:blocked" > /dev/null 2>&1 || true
fi

branch="autodev/issue-$issue-gen$generation"
if git show-ref --quiet "refs/heads/$branch"; then
  fail "$branch already exists locally; remove it before claiming this episode again"
fi

# An open pull request means this episode already delivered once, so the run
# becomes a correction against the failure its current head reports.
pull=$(gh pr list --repo "$repository" --head "$branch" --state open \
  --json number,headRefOid --jq '.[0] // empty')
correction=false
if [ -n "$pull" ]; then
  correction=true
  pull_number=$(jq -r '.number' <<< "$pull")
  head=$(jq -r '.headRefOid' <<< "$pull")
  failing=$(gh pr checks "$pull_number" --repo "$repository" --json name,state \
    --jq '[.[] | select(.state == "FAILURE" or .state == "ERROR" or .state == "TIMED_OUT" or .state == "CANCELLED") | .name] | sort | join(",")')
  [ -n "$failing" ] || { echo "pull request #$pull_number has no failing check to correct."; exit 0; }
  signature=$(printf '%s' "$failing" | shasum -a 256 | cut -d' ' -f1)
  budget=$(jq -r '.project_revision.projection.delivery.correction_budget' "$work/snapshot.json")
  scripts/autodev-comment-record.sh read "$repository" "$issue" \
    '<!-- autodev:attempts -->' attempts > "$work/attempts.json"
  used=$(jq --argjson g "$generation" \
    '[.attempts[] | select(.authorization_generation == $g)] | length' "$work/attempts.json")
  if [ "$used" -ge "$budget" ]; then
    gh issue edit "$issue" --repo "$repository" --add-label "autodev:human-needed" > /dev/null
    fail "correction budget of $budget is exhausted for generation $generation"
  fi
  repeated=$(jq -r --argjson g "$generation" --arg head "$head" --arg signature "$signature" \
    '[.attempts[] | select(.authorization_generation == $g)] | last
     | if . != null and .head == $head and .failure_signature == $signature then "true" else "false" end' \
    "$work/attempts.json")
  if [ "$repeated" = "true" ]; then
    gh issue edit "$issue" --repo "$repository" --add-label "autodev:human-needed" > /dev/null
    fail "the same failure repeated on an unchanged head; a correction would make no progress"
  fi
  echo "Correcting pull request #$pull_number, attempt $((used + 1)) of $budget, failing: $failing"
fi

git fetch --quiet origin
if [ "$correction" = true ]; then
  git worktree add --quiet -b "$branch" "$tree" "origin/$branch"
  {
    echo "# Failing checks"
    printf '%s' "$failing" | tr ',' '\n' | sed 's/^/- /'
    echo
    echo "# Log tail"
    run=$(gh run list --repo "$repository" --branch "$branch" --limit 1 --json databaseId --jq '.[0].databaseId // ""')
    [ -z "$run" ] || gh run view "$run" --repo "$repository" --log-failed 2> /dev/null | tail -120
  } > "$tree/.autodev-failure.md"
else
  git worktree add --quiet -b "$branch" "$tree" "origin/$base"
fi
cp "$work/agent-input.md" "$tree/.autodev-task.md"
{
  echo ".autodev-task.md"
  echo ".autodev-question.md"
  echo ".autodev-failure.md"
} >> "$(git -C "$tree" rev-parse --git-path info/exclude)"

prompt="$root/scripts/delivery-prompt.md"
if [ "$correction" = true ]; then
  prompt="$root/scripts/correction-prompt.md"
  if [ "$apply" = true ]; then
    # Recorded before the engine runs, so a crashed attempt still counts and
    # the budget cannot be spent in a loop.
    jq --argjson g "$generation" --argjson attempt "$((used + 1))" \
      --arg head "$head" --arg signature "$signature" \
      '.attempts + [{authorization_generation: $g, attempt: $attempt, head: $head, failure_signature: $signature}]' \
      "$work/attempts.json" > "$work/next-attempts.json"
    scripts/autodev-comment-record.sh write "$repository" "$issue" \
      "$(jq -r '.comment_id // ""' "$work/attempts.json")" \
      '<!-- autodev:attempts -->' "$work/next-attempts.json"
  fi
fi

echo "Running the engine for issue #$issue, generation $generation."
codex exec --sandbox workspace-write --cd "$tree" "$(cat "$prompt")" \
  2>&1 | tee "$work/engine.log"

# A missing decision suspends the episode instead of guessing. The suspension
# is recorded before the question is published, and the ready label is removed
# last, so an interrupted run can be repeated safely.
question="$tree/.autodev-question.md"
if [ -s "$question" ]; then
  echo "The engine needs a decision:"
  cat "$question"
  if [ "$apply" != true ]; then
    echo "Would suspend the episode, publish this question, and remove the ready label."
    echo "Re-run with --apply to perform these writes."
    exit 0
  fi
  jq -n \
    --argjson event_id "$(date +%s)" \
    --slurpfile episode "$work/episode.json" \
    '{kind: "needs_input_recorded",
      repository_id: $episode[0].episode.repository_id,
      event_id: $event_id,
      episode: $episode[0].episode,
      project_revision_sha256: $episode[0].project_revision_sha256,
      actor: null}' > "$work/needs-input-event.json"
  cargo run --locked --quiet -- --transition \
    --root . --event "$work/needs-input-event.json" --current "$work/episode.json" > "$work/decision.json"
  jq -s '.[0] + [.[1].authorization]
         | group_by(.episode.authorization_generation)
         | map(last)' "$work/prior.json" "$work/decision.json" > "$work/next.json"
  scripts/autodev-record-write.sh "$repository" "$issue" "$comment_id" "$work/next.json"
  gh issue comment "$issue" --repo "$repository" --body-file "$question" > /dev/null
  gh issue edit "$issue" --repo "$repository" --add-label "autodev:needs-input" > /dev/null
  gh issue edit "$issue" --repo "$repository" --remove-label "autodev:ready" > /dev/null
  echo "Suspended issue #$issue and published the question."
  exit 0
fi

scripts/autodev-changed-paths.sh "$tree" > "$work/changed.txt"
[ -s "$work/changed.txt" ] || fail "the engine produced no change"

protected_pattern=$(jq -r '.project_revision.projection.delivery.protected_paths[]' "$work/snapshot.json" \
  | scripts/autodev-protected-paths.sh)
protected=$(grep -E "$protected_pattern" "$work/changed.txt" || true)
if [ -n "$protected" ]; then
  echo "$protected" >&2
  gh issue edit "$issue" --repo "$repository" --add-label "autodev:human-needed"
  fail "the change touches protected paths"
fi

echo "Running project-owned checks."
( cd "$tree" && cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings \
  && cargo test --locked --all-targets && cargo run --locked --quiet -- . )

title=$(jq -r '.task_snapshot.projection.title' "$work/snapshot.json")
cat > "$work/pr-body.md" <<BODY
Delivered by the local runner for #$issue.

| Field | Value |
| --- | --- |
| Task digest | \`$(jq -r '.episode.task_sha256' "$work/episode.json")\` |
| Project revision | \`$(jq -r '.project_revision_sha256' "$work/episode.json")\` |
| Authorization generation | $generation |
| Engine | $(codex --version) |

Checks run before this pull request existed: \`cargo fmt --check\`, \`cargo clippy --locked --all-targets -- -D warnings\`, \`cargo test --locked --all-targets\`, and \`cargo run --locked --quiet -- .\`.

Closes #$issue
BODY

if [ "$apply" != true ]; then
  echo
  if [ "$correction" = true ]; then
    echo "Would push the correction to: $branch"
  else
    echo "Would push branch: $branch"
  fi
  echo "Would open a draft pull request titled: $title"
  sed 's/^/  changed: /' "$work/changed.txt"
  echo "Re-run with --apply to perform these writes."
  exit 0
fi

git -C "$tree" add -A
if [ "$correction" = true ]; then
  git -C "$tree" commit --quiet -m "Correct $title" \
    -m "Attempt $((used + 1)) for issue #$issue, generation $generation, against $failing."
else
  git -C "$tree" commit --quiet -m "$title" -m "Authorized issue #$issue, generation $generation."
fi
git -C "$tree" push --quiet -u origin "$branch"
pushed=true
if [ "$correction" = true ]; then
  echo "Updated pull request #$pull_number."
else
  gh pr create --repo "$repository" --base "$base" --head "$branch" --draft \
    --title "$title" --body-file "$work/pr-body.md"
fi
