---
status: proposed
date: 2026-08-15
---

# ADR 0006: Add an event-driven improvement loop

## Context

The first version proved that Autodev can reduce an idea to an approval-bound plan, execute one task through an Agent Host, and retain verification evidence. It does not continue without another chat request, create and maintain a pull request, react to CI or review feedback, or turn project outcomes into judged knowledge.

The existing GitHub task contract also binds approval to one complete Issue Graph. Adding an unrelated issue therefore invalidates every task in that graph. That boundary does not fit a backlog that changes while approved work is running.

The intended product now starts from a trusted issue authorization and continues through implementation, review, questions, and learning. This activates ADR 0001's runtime-orchestration upgrade trigger and requires a narrower approval unit than ADR 0004's complete graph projection.

## Decision

Keep the Autodev Skill as the planning and judgment contract, and add an event-driven GitHub adapter for durable execution. The first adapter uses [GitHub Agentic Workflows](https://github.github.com/gh-aw/) in GitHub Actions with Codex as the initial engine. The selected engine and model version are operational state inside the approved provider, data-use, and cost policy. A replacement engine must pass the same capability and permission checks, while changing the policy or approved tools requires a new project revision.

GitHub Issues, pull requests, checks, comments, and labels hold workflow state. Autodev does not add a daemon, queue, database, or long-running paused process.

The approved project revision contains the exact Overview bytes plus a semantic projection of the Task Source repository, authorizer and cancel roles, delivery base branch, protected paths, required checks and review rules, registered tool names, purposes, interfaces and permission envelopes, Knowledge source aliases and visibility, candidate carrier identity and visibility, writable Knowledge target identity and visibility, merge mode, correction budget, and engine policy. A Knowledge source without declared visibility is private. The engine policy binds allowed providers, data-use boundary, and cost class. Credential values, local Knowledge paths, the selected engine and model version within that policy, installation state, and authentication state remain operational.

Authorization has two levels:

- The approved project revision defines project intent, verification expectations, registered tools, Knowledge destination, and delegated automation boundaries.
- An authorized maintainer applying `autodev:ready` approves the exact task snapshot for one issue. The snapshot binds canonical raw GitHub identity, title, body, dependency identities, and the current project revision. The Agent receives a separate integrity-filtered projection, and evidence retains both digests. A changed task invalidates only that task. Adding or changing another issue does not.

The GitHub Task Source is one configured repository, without a required root issue. Issues become executable only through task-scoped authorization. Native blocking relationships are allowed only between issues in that repository. A dependency is complete for delivery only when its approved result has verified evidence and its pull request is merged into the dependent task's base. A dependency completion event re-evaluates already authorized blocked issues.

Cutover is a separate planning transition rather than an executable issue. The local Task Graph carries a machine-readable transition marker after task-scoped authorization, and the Skill and validator stop later task selection there. A later proposed revision creates issues only for remaining work, omits the completed local predecessor, presents the complete configuration projection and snapshots, and requests exact approval before switching Task Source and removing `tasks.yaml`.

Untrusted issue creation is input, not execution authority. Authorization requires an exact permitted actor role, the approval label, the raw task digest, a separate integrity-filtered Agent context, and an event-time task snapshot. Configure exact `on.roles`, `tools.github.min-integrity: approved`, approval labels, and refusal labels. Do not use a label command that removes authorization before validation. Later runs use the recorded actor and digest as authority. Human label removal revokes the episode; a workflow removal paired with the recorded `needs-input` transition suspends it instead.

Assign a monotonic authorization generation under per-issue concurrency and retain its durable authorization record. Replays of the same ready-label event reuse the generation; a new ready application after abandonment creates the next generation even when task content is unchanged. Use `repository + issue number + task digest + authorization generation` as the delivery episode identity. GitHub may start duplicate workflow runs, but each run must resolve to the same episode, deterministic branch and pull request marker. A duplicate becomes a no-op after reading current state. Do not force-push or allow a failed update to fall back to another pull request.

Allow one active episode and pull request per issue. An authorized task edit or approved project revision supersedes the active episode, invalidates its required gate, closes its unmerged pull request, and records the replacement digest before another snapshot can start. Issue closure, human removal of `autodev:ready`, an authorized cancel command, or human pull request closure without merge abandons the episode only for an actor with the configured cancel role. An untrusted edit or close may fail the exact task gate but cannot close a pull request, change labels, merge, or authorize cleanup. A recorded workflow transition to `needs-input` suspends the episode. Reopening requires a new ready authorization.

Make `merged` an absorbing terminal state. A later issue close is a no-op. A controller-closed superseded pull request retains `superseded` and links to its replacement digest instead of becoming abandoned. Insight judgment uses the complete linked episode lineage.

Preserve the event-time project and task snapshots as deterministic, non-Agent inputs. Delivery writes require an exact active snapshot. A separate restrictive cleanup gate validates the old episode identity and trusted invalidating event, then permits only gate failure, terminal evidence, and, for authorized cancellation or supersession, closing the old pull request. Re-read the applicable gate inputs immediately before every write. Initial validation alone is insufficient.

The GitHub adapter follows this lifecycle:

1. Validate project and task approval, then claim the delivery episode.
2. If a material decision, permission, or credential is missing, add one focused question, mark the issue `autodev:needs-input`, and end the run.
3. Otherwise run the configured Agent Host in an isolated branch, execute project-owned checks, and create or update one draft pull request through validated write operations.
4. Evaluate CI and actionable review findings against the approved task, applicable knowledge, and current code. Start a new correction run only when the finding is supported.
5. Stop with `autodev:human-needed` when a material planning change is required, a correction produces no new head or repeats the same normalized failure signature, or the episode's correction budget is exhausted. The first adapter allows the initial delivery plus two correction runs; only a newly authorized task snapshot resets that budget.
6. Mark the pull request ready for its configured merge policy only when deterministic checks on its current head pass. The agent's self-report is not a readiness signal.

For a question, persist the suspended transition first, then publish the question and `autodev:needs-input`, and remove `autodev:ready` last. Each step is idempotent. Material answers are incorporated into the issue body, `autodev:needs-input` is removed, and an authorized maintainer reapplies `autodev:ready` to create a new task snapshot. A credential or other operational unblock that does not change approved content resumes through an authorized command event bound to the existing digest.

Merge behavior is an approval-bound project policy. The Autodev dogfood project selects `auto_after_gates`. One trusted job publishes `autodev/gate` for the exact head and completes. A separate trusted merge job then revalidates the current task, required checks, branch freshness, mergeability, every unresolved review thread, GitHub review decision, configured approvals, questions, repository rules, and protected-file policy. It calls GitHub's merge API with the verified head SHA; a changed head or HTTP 409 fails closed. The repository-scoped delivery credential has no ruleset bypass. GitHub native auto-merge and the experimental Agentic Workflow merge output are not used. When any gate fails, the pull request remains ready for human action. The Agent never decides or performs the merge itself.

Agent-created pull requests require a repository-scoped delivery credential with only Contents, Pull requests, and Checks read and write access. It supplies the CI-trigger commit, task-bound gate, and deterministic merge but is never available to the Agent, pull request CI, or review-event receivers. Agent PR CI receives no secrets or write token and performs no deployment or release. Privileged controller jobs use trusted default-branch workflow definitions, read metadata only, and never execute pull request code or consume its artifacts.

Commit the human-readable Agentic Workflow source and its compiled lock workflow. Pin the compiler and actions, validate strictly, reject generated drift, and use staged output for the first live event. Protected-file changes stop as `human-needed`; disable fallback behavior that could create a second issue or pull request.

Use an explicit event matrix: an allowed `issues:labeled` event starts work; an allowed authorizer's edit may supersede and clean up the old episode; an allowed cancel actor's ready-label removal or closure may abandon and clean it up; untrusted invalidation only fails the exact gate; reopening waits for new authorization; a configured `workflow_run:completed` advances CI, Agent review, gate, or unprivileged receiver state without executing pull request code or artifacts in the privileged controller; review events run without secrets or write tokens; an authorized `issue_comment:created` command resumes an unchanged operational unblock or rechecks a human-resolved thread; a trusted `repository_dispatch` rechecks a controller-resolved thread; and pull request closure finalizes merge, supersession, or abandonment. Self-generated events outside those transitions are no-ops.

Project knowledge and delegated authority remain separate, so a preference or accepted lesson cannot grant repository permissions or expand the allowed action set.

Every run records a concise durable result keyed by episode and run attempt, including the task digest, commit and head SHA, check outcome, artifact digest, selected engine and version, and applied or rejected Knowledge revisions. Public evidence uses an opaque digest when a Knowledge source is more private, with the private identifier mapping retained only in the authorized candidate carrier. Workflow logs and expiring artifacts are supplemental evidence. Insight judgment waits for a task outcome such as merge, explicit abandonment, or exhausted escalation, then considers the complete linked episode lineage, including superseded attempts and rejected approaches.

An evidence-backed insight uses a configured candidate carrier whose visibility is at least as restrictive as every source used for it. A private carrier may receive a pending candidate automatically. For a public carrier, the autonomous run stores only an opaque review-needed record; candidate content is regenerated from durable evidence in an authorized review session and written only after a human approves sanitization and target visibility. Before later work, Autodev may compare relevant candidates with accepted knowledge, but pending candidates are hypotheses rather than authority.

Generalized knowledge may be promoted through a separately authorized adapter to one configured writable Knowledge target. For this project's dogfood that target is `syshin0116/dev-knowledge`. A human first approves the candidate's sanitization and target visibility because a public pull request exposes its content before merge. Judgment records applicability, provenance, freshness, and whether the candidate confirms, qualifies, contradicts, or supersedes existing knowledge. An agent may propose that change but may not approve its own promotion or use it to expand its delegation policy.

A later scheduled challenge loop first performs a deterministic scan for due `stale_after` values. Only due or directly contradicted records trigger current-source research. The result is a challenge candidate or knowledge pull request, never a silent rewrite.

## Considered options

### Use a hosted coding agent directly

Assigning an issue to a hosted coding agent provides the shortest issue-to-pull-request path. It does not by itself preserve Autodev's task approval, cross-engine boundary, evidence model, or Knowledge judgment.

### Build an Autodev server and worker queue

This would provide full control over scheduling and state but duplicates GitHub's event log, concurrency, permissions, and review surfaces before they have proved insufficient.

### Use GitHub Agentic Workflows as the first adapter

This is selected. It supports multiple Agent engines, read-only agent execution, validated write outputs, event triggers, and GitHub-native audit state. Its preview status is contained in an adapter rather than made part of the core knowledge contract.

## Consequences

- Autodev can continue after the planning conversation without requiring the user to reopen a local Agent session.
- Project-level changes can pause all tasks, while task edits invalidate only the affected issue revision.
- Questions and retries become separate event runs instead of suspended model sessions.
- GitHub Actions usage, engine credentials, and repository policy become operational prerequisites for the first adapter.
- Merge behavior follows the approval-bound project policy; the dogfood project permits a deterministic squash merge after all gates.
- Accepted knowledge can inform decisions, but only an explicit delegation policy authorizes actions.
- Other task systems and a custom runtime remain deferred until a real project demonstrates that the GitHub adapter is insufficient.

## References

- [GitHub Agentic Workflows: How workflows run](https://github.github.com/gh-aw/introduction/how-they-work/)
- [GitHub Agentic Workflows: Safe outputs](https://github.github.com/gh-aw/reference/safe-outputs/)
- [GitHub Agentic Workflows: Pull request outputs](https://github.github.com/gh-aw/reference/safe-outputs-pull-requests/)
- [GitHub Agentic Workflows: Triggering CI](https://github.github.com/gh-aw/reference/triggering-ci/)
- [GitHub Agentic Workflows: Integrity filtering](https://github.github.com/gh-aw/reference/integrity/)
- [GitHub Agentic Workflows: Compilation](https://github.github.com/gh-aw/reference/compilation-process/)
- [GitHub Agentic Workflows: Outcomes](https://github.github.com/gh-aw/reference/outcomes/)
- [GitHub Actions events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)
- [GitHub Copilot cloud agent issue flow](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/cloud-agent/use-cloud-agent-on-github)
- [Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
