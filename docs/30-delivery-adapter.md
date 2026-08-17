# Delivery Adapter

The first adapter turns one authorized issue into a draft pull request. It is two workflows with different trust levels, not one agent with write access.

- [Trust split](#trust-split)
- [Authorization record](#authorization-record)
- [Agent input](#agent-input)
- [Staged first event](#staged-first-event)
- [Not implemented yet](#not-implemented-yet)

## Trust split

`autodev-authorize.yml` is the trusted controller. It runs from the default branch on `issues: labeled`, holds `contents: read`, `issues: write`, and `actions: write`, and never executes pull request code. Every event value reaches its shell through the environment, so issue text cannot become script.

Its job is to decide, not to implement:

1. Resolve the actor's repository role through the collaborators API.
2. Read the approved task snapshot with `--print-task-snapshot`, which fails closed without the approved ready label.
3. Build the integrity-filtered agent input and the `ReadyEvent`.
4. Call `--authorize` with the prior authorization record.
5. Persist the returned record before any side effect.
6. Dispatch delivery only for a `start` or `replay` action. Anything else labels the issue `autodev:human-needed` and fails the run.

`autodev-deliver.md` is the agentic workflow, compiled to `autodev-deliver.lock.yml` by gh-aw v0.86.2. The agent job is read-only: `contents: read`, `issues: read`, `pull-requests: read`. It reaches the repository only through validated safe outputs, and its single output is one draft pull request with `protected-files: blocked` and no issue fallback. Both the source and the lock file are committed, every action and container is pinned by digest, and CI recompiles to reject drift.

## Authorization record

The durable record is one comment on the issue, marked with `<!-- autodev:authorization -->` and carrying a JSON array of authorization records in a fenced block. The controller upserts it and keeps the latest record per authorization generation.

The comment is a carrier, not authority. It holds no approval-bound planning content, so editing it cannot change what was approved; a tampered record produces a decision that fails its own gate. Per-issue serialization comes from the workflow concurrency group `autodev-episode-<issue>`, which never cancels a run in progress.

## Agent input

The controller writes the only task description the Agent sees. It contains the title, the body, and the blocking issue numbers from the snapshot projection. A body whose author association is outside `OWNER`, `MEMBER`, or `COLLABORATOR` is withheld with a stated reason rather than filtered in place, and the Agent is instructed to stop when it sees that.

The input travels as a base64 workflow input and is decoded to `/tmp/gh-aw/agent/autodev-task.md` before the Agent starts. The raw issue body and the filtered projection keep separate digests in the authorization record.

## Staged first event

`safe-outputs.staged: true` makes the first live event print its intended writes instead of performing them. Remove it only after a real event has produced the expected staged output.

Delivery also needs `CODEX_API_KEY` or `OPENAI_API_KEY` in repository secrets. Without it the agent job fails before doing anything.

## Not implemented yet

These belong to the remaining verification bullets on the delivery and controller issues:

- The repository-scoped delivery credential, the CI-trigger commit, `autodev/gate`, and the deterministic merge job. `autodev/gate` is deliberately absent from the base branch ruleset until something can publish it.
- Dependency blocking, questions and `autodev:needs-input`, review correction inside the correction budget, and the `--transition` events that suspend, supersede, or abandon an episode.
- Durable per-attempt evidence outside expiring workflow logs.
- Enforcing the approved `protected_paths` by name. The safe output currently blocks top-level dot folders, which covers `.autodev/**` and `.github/workflows/**`, but it is not yet derived from the approved project revision.
