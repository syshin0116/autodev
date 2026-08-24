# Delivery Adapter

The first adapter turns one authorized issue into a draft pull request. GitHub decides and remembers; the operator's machine implements.

- [Trust split](#trust-split)
- [Ending an episode](#ending-an-episode)
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

## Ending an episode

`autodev-transition.yml` ends episodes. It reacts to a merged or closed pull request, removal of the ready label, and issue closure, and decides every outcome through the Rust boundary.

Pull request closure arrives as `pull_request_target` so the workflow definition always comes from the default branch, and the workflow never checks out or runs pull request code.

- A merged pull request completes the episode as `merged`, which is absorbing. Completion requires the approved base branch and every required check passing on that pull request, so a merge that skipped the gates cannot claim success.
- Closing the issue is also how a merge reports itself, so an issue closure is a no-op when the episode's branch already has a merged pull request. Success does not abandon itself.
- Ready-label removal, issue closure, and closing the pull request unmerged abandon the episode when the actor holds a cancel role. Any other actor only fails that gate, and nothing is written.
- Abandonment then runs cleanup: close the open pull request, delete the episode branch, and record `cleanup_completed`. Only after that can reapplying the ready label start the next authorization generation, which is what makes a failed episode retryable.

Event identity is `run_id * 10 + kind`, so re-running a run replays one identity and becomes a no-op, while two kinds inside one run stay distinct.

## Authorization record

The durable record is one comment on the issue, marked with `<!-- autodev:authorization -->` and carrying a JSON array of authorization records in a fenced block. The controller upserts it and keeps the latest record per authorization generation.

`scripts/autodev-record-write.sh` is the only writer of that comment and `scripts/autodev-episode-record.sh` is the only reader, shared by both controllers and the runner, and `tests/episode_record.rs` covers a writer to reader round trip, a record on a later comment page, an issue with no record, and a damaged record that must fail instead of looking like a first authorization.

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

Work happens in a scratch git worktree under the system temp directory, so the operator's checkout is never disturbed. The worktree is released when the run ends, and a run that pushed nothing also deletes its branch, so the same episode can be claimed again. The engine log and the produced diff stay in the workspace, whose path is printed on exit.

## Not implemented yet

These belong to the remaining verification bullets on the delivery and controller issues:

- The repository-scoped delivery credential, the CI-trigger commit, `autodev/gate`, and the deterministic merge job. `autodev/gate` is deliberately absent from the base branch ruleset until something can publish it.
- Dependency blocking, questions and `autodev:needs-input`, review correction inside the correction budget, and the supersession path for an authorized task edit.
- Serialization between an issue event and a pull request event for the same episode. Their concurrency groups differ, so the library's event identity and status guards are what keep a racing pair from corrupting the record.
- Durable per-attempt evidence outside the local engine log.
- Deriving the protected paths from the approved project revision instead of hard-coding them in the runner.
