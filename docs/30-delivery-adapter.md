# Delivery Adapter

The first adapter turns one authorized issue into a draft pull request. GitHub decides and remembers; the operator's machine implements.

- [Trust split](#trust-split)
- [Ending an episode](#ending-an-episode)
- [Authorization record](#authorization-record)
- [Agent input](#agent-input)
- [Dependencies](#dependencies)
- [Questions](#questions)
- [Corrections](#corrections)
- [Running a delivery](#running-a-delivery)
- [Not implemented yet](#not-implemented-yet)

## Trust split

`autodev-authorize.yml` is the trusted controller. It runs from the default branch on `issues: labeled`, holds `contents: read` and `issues: write`, runs no engine, and writes no code. Every event value reaches its shell through the environment, so issue text cannot become script.

1. Resolve the actor's repository role through the collaborators API.
2. Read the authorized task snapshot with `--print-task-snapshot`, which fails closed without the configured ready label.
3. Build the integrity-filtered agent input and the `ReadyEvent`.
4. Call `--authorize` with the prior authorization record.
5. Persist the returned record before any side effect.
6. Report a claimable episode, or label the issue `autodev:human-needed` and fail for any other action.

`scripts/autodev-deliver.sh` is the local runner. It claims one recorded episode, runs the engine with the operator's subscription, and performs the branch, push, and pull request itself after the checks pass. The engine gets a working tree and the filtered projection, never a repository credential, and is told not to commit or push.

Delivery therefore advances only while the operator runs the command. [ADR 0007](../adr/0007-run-the-delivery-engine-on-a-local-host.md) records why.

## Ending an episode

`autodev-transition.yml` ends episodes. It reacts to a merged or closed pull request, an edited issue, removal of the ready label, and issue closure, and decides every outcome through the Rust boundary.

Pull request closure arrives as `pull_request_target` so the workflow definition always comes from the default branch, and the workflow never checks out or runs pull request code.

- A merged pull request completes the episode as `merged`, which is absorbing. Completion requires the configured base branch and every required check passing on that pull request, so a merge that skipped the gates cannot claim success.
- Closing the issue is also how a merge reports itself, so an issue closure is a no-op when the episode's branch already has a merged pull request. Success does not abandon itself.
- Ready-label removal, issue closure, and closing the pull request unmerged abandon the episode when the actor holds a cancel role. Any other actor only fails that gate, and nothing is written.
- An issue edit by an authorizer supersedes the episode. Cleanup then closes its pull request and deletes its branch, and reapplying `autodev:ready` records the replacement digests and starts the next generation in the same run.
- Abandonment then runs cleanup: close the open pull request, delete the episode branch, and record `cleanup_completed`. Only after that can reapplying the ready label start the next authorization generation, which is what makes a failed episode retryable.

Event identity is `run_id * 10 + kind`, so re-running a run replays one identity and becomes a no-op, while two kinds inside one run stay distinct.

## Authorization record

The durable record is one comment on the issue, marked with `<!-- autodev:authorization -->` and carrying a JSON array of authorization records in a fenced block. The controller upserts it and keeps the latest record per authorization generation.

`scripts/autodev-comment-record.sh` reads and writes every marked state comment, including the attempt record, with `scripts/autodev-episode-record.sh` and `scripts/autodev-record-write.sh` as the authorization-record wrappers used by both controllers and the runner, and `tests/episode_record.rs` covers a writer to reader round trip, a record on a later comment page, an issue with no record, and a damaged record that must fail instead of looking like a first authorization.

The comment is a carrier, not authority. Editing it cannot change committed planning or the authorized task snapshot; a tampered record produces a decision that fails its own gate. Per-issue serialization comes from the workflow concurrency group `autodev-episode-<issue>`, which never cancels a run in progress.

## Agent input

`scripts/autodev-agent-input.sh` builds the only task description the Agent sees: the title, the body, and the blocking issue numbers from the snapshot projection. A body whose author association is outside `OWNER`, `MEMBER`, or `COLLABORATOR` is withheld with a stated reason rather than filtered in place, and an association that could not be read aborts rather than guessing.

The controller and the runner both call that script, so the runner can compare its rebuild against the recorded `agent_input_sha256`. Changing the script's output changes that digest and invalidates every in-flight episode. `tests/agent_input.rs` pins the output.

## Dependencies

A task issue's native blocking relationships are its dependencies. The runner refuses to start while any of them is incomplete, and labels the issue `autodev:blocked`. The label is a report, not the state: `scripts/autodev-dependency-status.sh` derives each dependency's status from that dependency's own authorization record, so a hand-applied or hand-removed label changes nothing.

A dependency counts as complete only when its record reached the merged terminal state with the configured base branch. An authorized but unfinished dependency, and one nobody authorized at all, both get a status that fails the check, because the readiness rule requires exactly one status per declared dependency.

## Questions

When the engine cannot proceed without a decision the task does not contain, it writes `./.autodev-question.md` and changes nothing. The runner then records the suspension first, publishes the question as an issue comment, adds `autodev:needs-input`, and removes `autodev:ready` last. Every step is idempotent, so an interrupted run can be repeated.

Removing the ready label is what would normally abandon an episode. The transition controller treats that removal as bookkeeping when the episode is already recorded as suspended, which is why the suspension is persisted before the label moves. A human who wants to cancel a suspended episode closes the issue instead.

Answering means editing the issue body. That edit supersedes the episode, and reapplying `autodev:ready` starts the next generation against the new content.

## Corrections

When the episode's branch already has an open pull request, the run becomes a correction instead of a second delivery. The runner reads the pull request's failing checks, builds `.autodev-failure.md` from their names and the failed job's log tail, and runs the engine with the correction prompt against the existing branch.

Three things stop it, each recorded rather than inferred:

- the configured correction budget for that authorization generation is spent
- the same normalized failure signature appears again on an unchanged head, which means the last attempt made no progress
- no check is failing, in which case there is nothing to correct

The first two label the issue `autodev:human-needed`. Attempt count, head, and failure signature live in a separate `<!-- autodev:attempts -->` comment, written before the engine runs so a crashed attempt still counts and the budget cannot be spent in a loop.

Only a newly authorized task snapshot resets the budget, because attempts are keyed by authorization generation.

## Running a delivery

```sh
scripts/autodev-deliver.sh --issue 7          # stops after the checks and prints intended writes
scripts/autodev-deliver.sh --issue 7 --apply  # also pushes the branch and opens the draft pull request
```

The runner refuses before touching anything when the issue has no authorization record, the episode is not active, the task or project revision digest changed after authorization, the rebuilt agent input does not match, or the branch already has an open pull request. A change touching a protected path labels the issue `autodev:human-needed` and stops. The list comes from the current project revision, not from the runner, so protecting one more path is a configuration change. `scripts/autodev-protected-paths.sh` turns those globs into the matching expression and `tests/protected_paths.rs` pins the conversion.

The planning files are protected for a specific reason. Now that the committed plan is the plan, whoever can commit it can approve it, so the Agent is kept out of the Project Overview and the decision records. A human still edits them normally.

Work happens in a scratch git worktree under the system temp directory, so the operator's checkout is never disturbed. The worktree is released when the run ends, and a run that pushed nothing also deletes its branch, so the same episode can be claimed again. The engine log and the produced diff stay in the workspace, whose path is printed on exit.

## Not implemented yet

These belong to the remaining verification bullets on the delivery and controller issues:

- The repository-scoped delivery credential, the CI-trigger commit, `autodev/gate`, and the deterministic merge job. `autodev/gate` is deliberately absent from the base branch ruleset until something can publish it.
- Reacting to review comments. Only failing checks drive a correction today.
- Starting a correction automatically. The operator runs the delivery command again, which is the same boundary as the first delivery.
- Re-evaluating an already authorized blocked issue when its dependency merges. Today the operator runs the delivery command again.
- Serialization between an issue event and a pull request event for the same episode. Their concurrency groups differ, so the library's event identity and status guards are what keep a racing pair from corrupting the record.
- Durable per-attempt evidence outside the local engine log.
