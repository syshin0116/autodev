# Delivery Adapter

The first adapter turns one authorized issue into a draft pull request. GitHub decides and remembers; the operator's machine implements.

- [Trust split](#trust-split)
- [Authorization record](#authorization-record)
- [Agent input](#agent-input)
- [Running a delivery](#running-a-delivery)
- [Not implemented yet](#not-implemented-yet)

## Trust split

`autodev-authorize.yml` is the trusted controller. It runs from the default branch on `issues: labeled`, holds `contents: read` and `issues: write`, runs no engine, and writes no code. Every event value reaches its shell through the environment, so issue text cannot become script.

1. Resolve the actor's repository role through the collaborators API.
2. Read the approved task snapshot with `--print-task-snapshot`, which fails closed without the approved ready label.
3. Build the integrity-filtered agent input and the `ReadyEvent`.
4. Call `--authorize` with the prior authorization record.
5. Persist the returned record before any side effect.
6. Report a claimable episode, or label the issue `autodev:human-needed` and fail for any other action.

`scripts/autodev-deliver.sh` is the local runner. It claims one recorded episode, runs the engine with the operator's subscription, and performs the branch, push, and pull request itself after the checks pass. The engine gets a working tree and the filtered projection, never a repository credential, and is told not to commit or push.

Delivery therefore advances only while the operator runs the command. [ADR 0007](../adr/0007-run-the-delivery-engine-on-a-local-host.md) records why.

## Authorization record

The durable record is one comment on the issue, marked with `<!-- autodev:authorization -->` and carrying a JSON array of authorization records in a fenced block. The controller upserts it and keeps the latest record per authorization generation.

The comment is a carrier, not authority. It holds no approval-bound planning content, so editing it cannot change what was approved; a tampered record produces a decision that fails its own gate. Per-issue serialization comes from the workflow concurrency group `autodev-episode-<issue>`, which never cancels a run in progress.

## Agent input

`scripts/autodev-agent-input.sh` builds the only task description the Agent sees: the title, the body, and the blocking issue numbers from the snapshot projection. A body whose author association is outside `OWNER`, `MEMBER`, or `COLLABORATOR` is withheld with a stated reason rather than filtered in place, and an association that could not be read aborts rather than guessing.

The controller and the runner both call that script, so the runner can compare its rebuild against the recorded `agent_input_sha256`. Changing the script's output changes that digest and invalidates every in-flight episode. `tests/agent_input.rs` pins the output.

## Running a delivery

```sh
scripts/autodev-deliver.sh --issue 7          # stops after the checks and prints intended writes
scripts/autodev-deliver.sh --issue 7 --apply  # also pushes the branch and opens the draft pull request
```

The runner refuses before touching anything when the issue has no authorization record, the episode is not active, the task or project revision digest changed after authorization, the rebuilt agent input does not match, or the branch already has an open pull request. A change touching `.autodev/**` or `.github/workflows/**` labels the issue `autodev:human-needed` and stops.

Work happens in a scratch git worktree under the system temp directory, so the operator's checkout is never disturbed. The engine log stays there.

## Not implemented yet

These belong to the remaining verification bullets on the delivery and controller issues:

- The repository-scoped delivery credential, the CI-trigger commit, `autodev/gate`, and the deterministic merge job. `autodev/gate` is deliberately absent from the base branch ruleset until something can publish it.
- Dependency blocking, questions and `autodev:needs-input`, review correction inside the correction budget, and the `--transition` events that suspend, supersede, or abandon an episode. Until those exist, a failed episode can only be retried by removing its authorization record by hand.
- Durable per-attempt evidence outside the local engine log.
- Deriving the protected paths from the approved project revision instead of hard-coding them in the runner.
