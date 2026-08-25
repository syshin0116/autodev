---
id: autodev
status: proposed
approval: pending
---

# Autodev Project Overview

## Background

The first Autodev milestone proved knowledge-aware planning, exact-revision approval, task execution through an Agent Host, and separate verification evidence. It still depends on the user returning to a chat and asking for each task. It does not operate a pull request, react to CI and review feedback, or judge accumulated outcomes against existing knowledge.

The intended product is a continuing project improvement loop. A user should be able to authorize a well-defined issue and return when Autodev has either prepared a reviewed pull request or found a decision that requires the user.

## Goal

Turn approved project intent and authorized tasks into verified pull requests, ask only when a material decision or permission is missing, and feed evidence-backed insights into a reviewable Knowledge judgment loop.

## Users

People using an Agent Skills compatible Agent Host who want knowledge-aware planning for any project and autonomous improvement for repository-based work, while retaining control over intent, repository writes, and reusable knowledge.

## Inputs

- A canonical Project Overview and project delegation policy
- Authorized issues with outcomes, planning references, dependencies, and checks
- User-owned Markdown Knowledge Roots
- Current project code, CI, review feedback, and official external sources
- Approved engine policy, project-selected Agent engine, and tools

## Deliverables

- Approval-bound project intent and task snapshots
- Draft pull requests with project-owned verification results
- Durable question, retry, and review state in GitHub
- Raw execution evidence, including failed and rejected approaches
- Pending insight and Knowledge challenge candidates with provenance

## Flow

```text
Planning
rough idea -> knowledge and current-source research -> focused interview
           -> Project Overview -> task issue -> authorization

Delivery
authorized issue -> validate and claim -> implement -> checks -> draft PR
                 -> review and fix -> configured merge policy
                 -> ask and stop when human judgment is required

Learning
issue + runs + PR + CI + review -> evidence -> insight candidate
                                -> Knowledge judgment
                                -> confirm, qualify, contradict, or supersede

Challenge
due or contradicted Knowledge -> current-source research
                              -> challenge candidate or Knowledge PR
```

## Decisions

### Product boundary

- Keep one portable Autodev Skill for planning and judgment. The Skill is not a daemon or model runtime.
- Add event-driven delivery as a replaceable adapter. GitHub Actions holds authorization and the deterministic gates, while the engine runs on an operator-invoked local host with its existing subscription. The selected engine and model version are operational state inside the approved provider, data-use, and cost policy.
- Accept that implementation advances only while the operator runs the delivery command. GitHub holds durable episode state between runs, so an unavailable machine delays work instead of losing it.
- Use GitHub Issues, pull requests, checks, comments, and labels as durable workflow state. Do not add a queue, Task database, or paused model process.
- Keep the core vocabulary provider-neutral. Support Kaneo as the first external conversational Task Source required by a real project, while retaining GitHub as the only autonomous delivery adapter.
- Preserve general planning and conversational execution for non-software work. The first autonomous delivery adapter applies only to repository-based work.

### Delegation policy

- Allow an authorized `autodev:ready` event to create or reuse a task branch, run repository checks, create or update one draft pull request, and maintain Autodev labels and comments through validated outputs.
- Set this project's merge mode to `auto_after_gates`. A deterministic controller may perform a squash merge after every approved gate passes; the Agent cannot merge or waive a gate. Do not use GitHub native auto-merge or the experimental Agentic Workflow merge output for this milestone.
- Deny deployment, release, secret changes, destructive operations, and unrelated repository writes in the first autonomous milestone.
- Require a user decision for material scope or verification changes, new external cost, missing credentials, sensitive data exposure, or any action outside the approved task and this policy.
- Allow one initial delivery and two automated correction runs per task snapshot. Stop earlier when a correction produces no new head or repeats the same normalized failure signature. Only a newly authorized task snapshot resets the budget.
- Keep this policy inside the approval-bound Project Overview. Knowledge records may inform its wording but cannot modify or bypass it.

### Authorization and task state

- Define the project revision as the exact Overview bytes plus a semantic projection of Task Source repository, authorizer and cancel roles, delivery base branch, protected paths, required checks and review rules, registered tool names, purposes, interfaces and permission envelopes, Knowledge source aliases and visibility, candidate carrier identity and visibility, writable Knowledge target identity and visibility, merge mode, correction budget, and engine policy. Treat a Knowledge source without declared visibility as private. The engine policy binds allowed providers, data-use boundary, and cost class. Exclude credential values, local Knowledge paths, the selected engine and model version within that policy, installation state, and authentication state.
- Treat public or untrusted issue creation as input only. An authorized maintainer applying `autodev:ready` approves one exact task snapshot against the current project revision.
- Bind a task snapshot to canonical raw GitHub issue identity, title, body, dependency identities, and the project revision. Give the Agent a separate integrity-filtered projection and record both digests. A task edit invalidates only that task. Adding or changing another issue does not.
- Record the snapshot digest, authorizing actor, authorization event identity and generation, and project revision with the workflow run and resulting pull request or evidence.
- Configure one GitHub repository as the Task Source without requiring a root issue. A native blocking edge is valid only when both endpoints are issues in that repository.
- Configure one Kaneo MCP server, workspace, and project as an alternative Task Source. Bind task ID, number, title, description, and native relations in a canonical projection supplied by a fresh complete Host read. Exclude task status and other progress metadata. Do not keep a local task mirror.
- Treat a dependency as complete only when its approved result has verified evidence and its pull request is merged into the dependent task's base. Dependency completion re-evaluates already authorized blocked issues.
- Keep execution progress outside approval-bound content. Labels, pull request state, comments, checks, and evidence may change without rewriting the task.
- Assign a monotonic authorization generation under per-issue concurrency and retain its durable authorization record. Replays of the same ready-label event reuse the generation; a new ready application after abandonment creates the next generation even when task content is unchanged.
- Key a delivery episode by repository, issue number, task digest, and authorization generation. Duplicate runs may start, but every side effect must resolve to the same deterministic branch and pull request marker or become a no-op. Never force-push or fall back to a second pull request.
- Allow only one active delivery episode and pull request per issue. An authorized task edit or approved project revision supersedes the active episode, invalidates its required gate, closes its unmerged pull request, and records the replacement digest before another snapshot can start. Preserve the linked episode lineage for later judgment.
- Treat issue closure, human removal of `autodev:ready`, an authorized cancel command, or human pull request closure without merge as abandonment only when the actor has the configured cancel role. An untrusted edit or close may fail the task gate but cannot close a pull request, change labels, merge, or authorize cleanup. A workflow removal paired with the recorded `needs-input` transition suspends rather than abandons the episode. Reopening never resumes work until an authorized maintainer applies `autodev:ready` to a new snapshot.
- Make `merged` an absorbing terminal state. A later issue close is a no-op and cannot reverse successful evidence or dependency release. A controller-closed superseded pull request retains `superseded` rather than being reclassified as abandoned.
- Preserve event-time project and task snapshots as non-Agent inputs. Delivery writes require an exact active snapshot. A separate restrictive cleanup gate validates the old episode identity and trusted invalidating event, then permits only gate failure, terminal evidence, and, for an authorized cancel or supersession, closing the old pull request. Re-read the applicable gate inputs immediately before every write.
- Keep Knowledge and authority separate. A preference or accepted lesson can inform a decision but cannot grant write access, merge permission, budget, or a wider action scope.

Task-source cutover is a planning transition, not an executable issue. The local Task Graph carries a machine-readable transition marker after `task-scoped-authorization`; the Skill and validator must stop later task selection at that marker. After the first task is verified, create issues only for the remaining work, omit the completed local predecessor, present the complete config projection and task snapshots, and request exact approval. That approval switches configuration to GitHub Issues and removes `tasks.yaml`; no cutover issue or writable mirror remains.

### Delivery and questions

- Give the Agent the integrity-filtered projection and a working tree, never a repository credential. The runner performs the branch, push, and pull request after project-owned checks pass, so an Agent claim is never itself a write.
- Commit the local runner and keep its inputs deterministic. It claims one authorized episode from the durable record, refuses a task whose digest no longer matches, and prints its intended writes before performing them on a first run.
- Require project-owned checks before a pull request can become ready. Agent pull request CI has `contents: read`, receives no secrets or write token, and performs no deployment or release. A repository without this baseline CI must establish it before autonomous feature issues become ready.
- Use a repository-scoped delivery credential with only Contents, Pull requests, and Checks read and write access. It supplies the extra commit that starts CI, publishes the task-bound gate, and performs the deterministic merge, but is never available to the Agent, pull request CI, or review-event receivers. Privileged controller jobs run only from trusted default-branch workflow definitions, read metadata, and never execute pull request code or consume its artifacts.
- Create or update one draft pull request for an authorized task. Never create parallel pull requests for retries of the same task revision.
- Block protected-file changes in the first milestone and end the episode as `human-needed`. Disable any fallback that could create another issue or pull request.
- Review each current pull request head in a fresh run. Treat review findings as hypotheses until supported by the approved task, applicable Knowledge, current code, or executable checks.
- Automatically correct supported findings within the approved correction budget. Store attempt count, last processed head, and normalized failure signature with the delivery episode. Process only the configured event transitions; other self-generated events are no-ops.
- Ask one focused question and end the run when user judgment is required. Persist the suspended transition first, then publish the question and `autodev:needs-input`, and remove `autodev:ready` last. Each step is idempotent. Incorporate a material answer into the issue body, then require an authorized maintainer to reapply `autodev:ready`. Resume a credential or other non-content unblock through an authorized command bound to the unchanged task digest.
- Determine readiness from the current head SHA, CI, conflicts, every unresolved review thread, GitHub review decision, configured required approvals, and protected-file state. A thread that the integration cannot resolve moves the episode to `human-needed`. Do not use an Agent's confidence or completion claim as evidence.
- Publish an approval-bound `autodev/gate` check for the exact current head in one trusted job. Only after that job completes may a separate trusted merge job repeat the write-time validation and call GitHub's merge API with the verified head SHA. A changed head or HTTP 409 fails closed. The repository-scoped delivery credential has no ruleset bypass. If any gate fails or repository rules do not permit the merge, leave the pull request ready for human action.

The first adapter reacts only to this event matrix:

| Event | Accepted use |
| --- | --- |
| `issues:labeled` | Authorize only when an allowed actor applies `autodev:ready` to an open issue. Authorization records the episode; the operator's runner performs the delivery. |
| `issues:edited` | An allowed authorizer may supersede and clean up the old episode; an untrusted edit only fails its gate. |
| `issues:unlabeled`, `issues:closed` | An allowed cancel actor may abandon and clean up the episode; an untrusted event only fails its gate. |
| `issues:reopened` | Wait for a new ready authorization. |
| `workflow_run:completed` | Continue only for configured CI, Agent review, gate, or unprivileged event-receiver workflows, repository, branch, and head SHA. Never execute pull request code or artifacts in the privileged controller. |
| `pull_request_review` and `pull_request_review_comment` changes | Run a receiver with no secret or write token; a trusted default-branch `workflow_run` controller then re-reads review metadata. |
| Authorized `issue_comment:created` command | Resume an operational unblock, or recheck review gates after a human resolves a thread, bound to the unchanged task digest and head. |
| Trusted `repository_dispatch: autodev-recheck` | Recheck after the controller resolves a thread because GitHub emits no thread-resolved Actions event. |
| `pull_request:closed` | Finalize a merge, supersession, or abandonment, then release dependencies only for merged verified evidence. |

Configure exact `on.roles`, `tools.github.min-integrity: approved`, `approval-labels`, and `refusal-labels`. Do not use a label command that removes authorization before validation. Later runs trust the recorded authorizing actor and digest rather than label presence alone. Human label removal revokes the episode; the recorded workflow transition to `needs-input` is the only removal that preserves it in a suspended state.

### Evidence, insights, and Knowledge

- Record a concise durable result for every successful, failed, rejected, superseded, abandoned, and interrupted run. Include episode and attempt identity, task digest, commit and head SHA, check outcome, artifact digest, selected engine and version, and applied or rejected Knowledge revisions. If project evidence is more public than a Knowledge source, store only an opaque digest there and keep the private identifier mapping in the authorized candidate carrier. Treat expiring workflow logs and artifacts as supplemental evidence.
- Create an insight only when evidence supports a non-obvious claim with a stated context and applicability.
- Wait for a task outcome such as merge, explicit abandonment, or exhausted escalation before final insight judgment. Re-evaluate provisional observations against the complete linked episode lineage, including superseded attempts, questions, reviews, and rejected approaches.
- Compare an insight with existing Knowledge and classify it as confirming, qualifying, contradicting, superseding, duplicate, or unrelated.
- Treat pending insights as hypotheses. Accepted Knowledge may guide later work only when its applicability matches, it is not stale, its current sources remain valid, and no unresolved challenge conflicts with it.
- Configure a candidate carrier before autonomous learning. Its visibility must be at least as restrictive as every source used for the insight. A private carrier may receive the pending candidate automatically. For a public carrier, the autonomous run stores only an opaque review-needed record; candidate content is regenerated from durable evidence in an authorized review session and written only after a human approves its sanitization and target visibility. A separately authorized Knowledge adapter may then promote it to one configured writable target. This project's dogfood target is `syshin0116/dev-knowledge`; the Agent proposing a record cannot approve its own promotion.
- Preserve project-local decisions in the project. Promote only reusable reasoning, preferences, constraints, or lessons to the external Knowledge Root.
- In a later milestone, scan `stale_after` deterministically on a schedule. Run Web or official-source research only for due records or direct contradictions, and propose a challenge instead of silently rewriting Knowledge.
- Keep canonical Knowledge as human-readable Markdown. A graph database may later index relationships, but it is never the authority or the decision maker.

### Configuration and portability

- Store project paths, Task Source, selected engine and engine policy, authorizer and cancel roles, delivery base branch, protected paths, required checks and review rules, approved correction budget, merge mode, stable Knowledge source aliases and visibility, candidate carrier, writable Knowledge target, tool purposes, interfaces, and permission envelopes in `.autodev/config.yaml`. Keep credential values, authentication state, machine-specific Knowledge paths, and discovery outside tracked configuration.
- Verify a required CLI, MCP server, or other interface before relying on it. Do not silently install, authenticate, or substitute a tool.
- Let Agent Hosts and tools describe their general capabilities. Project configuration records only the choice and purpose for this project.
- Reuse templates from selected read-only Knowledge Roots after checking applicability, provenance, and mutable guidance against current official sources.

These boundaries are recorded in [ADR 0006](../adr/0006-add-an-event-driven-improvement-loop.md), which activated ADR 0001's runtime upgrade trigger and superseded ADR 0004's whole-Issue-Graph approval boundary. [ADR 0007](../adr/0007-run-the-delivery-engine-on-a-local-host.md) narrows where its engine runs.

## Success criteria

- Applying `autodev:ready` as an authorized maintainer creates one delivery episode for the approved issue revision. Duplicate runs produce no duplicate side effect.
- Reauthorizing an abandoned issue with unchanged content creates exactly one new authorization generation and preserves the prior terminal evidence.
- An untrusted issue or stale edit cannot write code, close a pull request, change labels, merge, or authorize cleanup. It may only cause the exact stale task gate to fail.
- Cancellation by an allowed actor or superseding an authorized revision stops the active episode, invalidates its gate, closes its unmerged pull request, and cannot release a dependency. An untrusted edit only fails the task gate, while a recorded workflow transition to `needs-input` suspends it.
- Adding an unrelated issue does not invalidate or restart an approved task.
- A blocked dependency does not run. Merged verified dependency completion re-evaluates and resumes an already authorized dependent issue.
- A missing material decision produces one focused question and a durable `needs-input` state. A material answer changes the issue body and requires new task authorization; an operational unblock resumes only through an authorized command.
- A ready task produces one draft pull request, runs project-owned CI, receives a fresh review, and applies supported findings within a bounded loop.
- Duplicate delivery events, retries, and workflow restarts do not create duplicate branches, pull requests, evidence, or insight candidates.
- Pull request readiness is calculated from the current head and repository state rather than Agent self-report.
- With `auto_after_gates`, a deterministic controller performs a squash merge only after the current task, head, CI, every review thread, review decision, required approval, question, ruleset, and protected-file gate passes.
- Successful and failed terminal runs can produce traceable evidence, while unsupported or obvious observations produce no insight.
- A pending insight can be compared with Knowledge but cannot become accepted authority or expand automation permissions without external approval.
- A promoted Knowledge record retains context, applicability, evidence, source revision, creation date, verification date, and freshness information.
- The GitHub adapter can change Agent engine within the approved provider, data-use, and cost policy without changing the Project Overview, task snapshot, evidence, or Knowledge contracts.

## Non-goals for the first autonomous milestone

- Deployment or release automation
- Task adapters for GitLab, Linear, Jira, or providers other than GitHub and Kaneo
- A custom Autodev server, queue, scheduler, dashboard, or notification service
- Continuous background model sessions
- Automatic promotion of insight candidates or self-expansion of Agent authority
- A hosted Knowledge service, graph database, embedding pipeline, or custom retrieval engine
- Automatic opportunity discovery or broad trend monitoring
- Multi-repository project task execution beyond one explicitly configured Knowledge contribution target
- Prescribed multi-agent personas or a mandatory multi-model review gauntlet

## Later milestone

After the issue-to-insight loop is dogfooded, add a deterministic `stale_after` scan. Only due or directly contradicted Knowledge should trigger current-source research, and the result must be a reviewable challenge rather than an automatic rewrite.

## References

- [ADR 0001: Keep the first version thin](../adr/0001-thin-first-version.md)
- [ADR 0004: Use GitHub Issues for project tasks](../adr/0004-use-github-issues-for-project-tasks.md)
- [ADR 0006: Add an event-driven improvement loop](../adr/0006-add-an-event-driven-improvement-loop.md)
- [ADR 0007: Run the delivery engine on a local host](../adr/0007-run-the-delivery-engine-on-a-local-host.md)
- [Reference workflow findings](research/reference-workflows.md)
- [GitHub Agentic Workflows](https://github.github.com/gh-aw/)
- [GitHub Agentic Workflows safe outputs](https://github.github.com/gh-aw/reference/safe-outputs/)
- [GitHub Agentic Workflows CI triggering](https://github.github.com/gh-aw/reference/triggering-ci/)
- [GitHub Actions events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)
- [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
- The [archived Autodev repository](https://github.com/syshin0116/autodev-archive) remains historical evidence rather than an active implementation base.

## Open questions

None that changes the proposed first autonomous milestone or its verification.
