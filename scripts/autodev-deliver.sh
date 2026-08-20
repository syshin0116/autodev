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
while [ $# -gt 0 ]; do
  case "$1" in
    --issue) issue=${2:?issue number}; shift 2 ;;
    --apply) apply=true; shift ;;
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

export STATE_MARKER='<!-- autodev:authorization -->'
repository=$(sed -n '/^task_source:/,/^[a-z]/p' .autodev/config.yaml | sed -n 's/^  repository: //p')
[ -n "$repository" ] || fail "config has no github_issues task source"

work=$(mktemp -d "${TMPDIR:-/tmp}/autodev-episode-XXXXXX")
trap 'echo "episode workspace: $work"' EXIT

# The durable record is authority for what was authorized. Rebuilding the
# snapshot and the agent input here is what catches a task edited after
# authorization.
gh api "repos/$repository/issues/$issue/comments" --paginate --slurp \
  --jq '[.[][] | select(.body | startswith(env.STATE_MARKER))] | last // {}' > "$work/state-comment.json"
[ "$(jq -r '.id // ""' "$work/state-comment.json")" != "" ] \
  || fail "issue #$issue has no authorization record; apply autodev:ready first"
jq -r '.body' "$work/state-comment.json" | sed -n '/^```json$/,/^```$/p' | sed '1d;$d' > "$work/record.json"
jq 'max_by(.episode.authorization_generation)' "$work/record.json" > "$work/episode.json"

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

branch="autodev/issue-$issue-gen$generation"
if [ -n "$(gh pr list --repo "$repository" --head "$branch" --state open --json number --jq '.[].number')" ]; then
  fail "$branch already has an open pull request; the correction loop is not implemented yet"
fi

tree="$work/tree"
git fetch --quiet origin
git worktree add --quiet -b "$branch" "$tree" origin/main
cp "$work/agent-input.md" "$tree/.autodev-task.md"
echo ".autodev-task.md" >> "$(git -C "$tree" rev-parse --git-path info/exclude)"

echo "Running the engine for issue #$issue, generation $generation."
codex exec --sandbox workspace-write --cd "$tree" "$(cat "$root/scripts/delivery-prompt.md")" \
  2>&1 | tee "$work/engine.log"

changed=$(git -C "$tree" status --porcelain | awk '{print $NF}')
[ -n "$changed" ] || fail "the engine produced no change"

protected=$(printf '%s\n' "$changed" | grep -E '^(\.autodev/|\.github/workflows/)' || true)
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
  echo "Would push branch: $branch"
  echo "Would open a draft pull request titled: $title"
  printf '%s\n' "$changed" | sed 's/^/  changed: /'
  echo "Re-run with --apply to perform these writes."
  exit 0
fi

git -C "$tree" add -A
git -C "$tree" commit --quiet -m "$title" -m "Authorized issue #$issue, generation $generation."
git -C "$tree" push --quiet -u origin "$branch"
gh pr create --repo "$repository" --base main --head "$branch" --draft \
  --title "$title" --body-file "$work/pr-body.md"
