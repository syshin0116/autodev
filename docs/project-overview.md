---
id: autodev
status: approved
approval: user-approved-in-chat-2026-08-10
---

# Autodev Project Overview

## Background

Agent Hosts can execute work, but a conversation rarely preserves the user's prior decisions, the reasoning behind them, or the exact plan they approved. Existing specification workflows cover parts of this path, yet commonly add overlapping documents, their own runtime, or software-only assumptions. The [reference review](research/reference-workflows.md) records the evidence behind this assessment.

## Goal

Turn an opportunity or rough idea into a concise, decision-complete Project Overview through knowledge-aware interviewing. Derive a verifiable Task Graph, obtain approval bound to those exact planning artifacts, then let the existing Agent Host execute and return evidence-backed learning candidates.

## Users

People using an Agent Skills compatible Agent Host who want to apply their own private, reusable knowledge to software and non-software projects.

## Inputs

- Free-form conversation
- Supplied opportunities, source files, and links
- User-owned Markdown knowledge roots outside autodev
- Current official sources for mutable technical decisions

## Deliverables

- One canonical Project Overview
- An approved Task Graph with dependencies and completion checks in the selected Task System of Record
- Execution artifacts and verification evidence
- Reviewable learning candidates

## Flow

```text
knowledge lookup + current research -> interview -> overview -> tasks -> approve -> host execution -> verify -> propose learnings
```

The first milestone ends at the approved handoff. Execution and learning reuse the Agent Host instead of introducing an autodev runtime.

## Decisions

- Package autodev as one portable Agent Skill. The Agent Host owns execution strategy.
- Keep user knowledge outside the distributed autodev repository in user-owned Markdown roots.
- Keep reusable implementation templates in selected Knowledge Roots as sourced Markdown records with linked assets. Copy and adapt them into projects without writing back to the root.
- Treat graph databases and other search indexes as rebuildable views over canonical Markdown and linked assets. Preserve the source path and commit for every indexed record.
- Record template creation, meaningful update, verification, and staleness separately. Check mutable claims against current official sources before every reuse because a prior decision is not current authority.
- For software projects without adequate CI, derive baseline CI after the stack and clean-checkout checks are known and before independent feature tasks become ready.
- Keep the Project Overview canonical. Add a Decision Record only when the alternatives and rationale will matter later.
- Store user-facing project artifacts at visible, configured paths. Reserve `.autodev/` for machine-readable configuration and state.
- Use GitHub Issues in `syshin0116/autodev` as this project's Task System of Record after an approved migration task and a separately approved cutover revision.
- Use one configured root issue as a non-executable plan container. Its recursive sub-issues in `syshin0116/autodev` are tasks, and native blocking relationships must stay within that membership. Reject pull requests, cross-repository tasks, and external dependency endpoints.
- Bind approval to a deterministic projection of issue identity, title, body, hierarchy, order, and dependencies. Exclude comments, assignees, labels, and open or closed state so normal execution does not invalidate approval.
- Fail closed when the complete approved Issue Graph cannot be read. A changed planning field, membership, order, or dependency reopens approval.
- Treat the approved Overview and Task Graph as an immutable planning revision. Store execution status and evidence separately so normal progress does not invalidate approval.
- Bind approval to the exact planning revision. Any included projection change requires approval of the new digest; reopen the interview only when the change is material.
- Treat approved decisions as accepted within the project only. A reusable lesson still enters the writable Knowledge Root as a candidate.
- Propose only sourced learning candidates with context and applicability. Check accepted, pending, deferred, and dismissed records before proposing another.
- Review new candidates in one batch at project close. A deferred candidate remains searchable and may resurface during later relevant work without blocking closure.
- Present the minimum decision-relevant summary first, then link to Decision Records and raw evidence.

These boundaries are accepted in [ADR 0001](../adr/0001-thin-first-version.md).
The proposed CI template boundary is recorded in [ADR 0003](../adr/0003-keep-ci-templates-in-user-knowledge.md).
The proposed task-source migration is recorded in [ADR 0004](../adr/0004-use-github-issues-for-project-tasks.md).

## Success criteria

- A free-form idea or supplied opportunity can reach an Overview with no unresolved question that could change scope, dependencies, or verification.
- Prior knowledge informs the interview and is cited without silently becoming binding.
- The Overview contains only information that changes a decision, action, constraint, or verification result.
- Every task links to the relevant Overview section or Decision Record and has a runnable or human-verifiable completion check.
- Execution is blocked when approval is missing, the Overview changes, or the approved Issue Graph projection changes.
- GitHub pagination is exhausted before validation. Pull requests, cross-repository tasks, external dependency endpoints, and unavailable or partial graph reads block execution.
- Execution evidence can accumulate without changing the approved planning revision.
- A completed run can produce a sourced learning candidate without promoting it automatically.
- A matching CI template is adopted only after its applicability and mutable claims are checked against the target project and current official sources.
- Generated baseline CI runs project-owned verification with least privilege and an explicit update path for remote dependencies.
- The same Skill passes the approval boundary in at least two Agent Hosts without changing the core package.
- One non-software fixture reaches approval without code-specific assumptions.

## Non-goals for the first version

- Automatic opportunity discovery
- A custom model runtime, server, chat UI, or Task database
- A hosted knowledge service, graph database, embedding pipeline, or custom retrieval engine
- Task integrations beyond GitHub Issues
- Prescribed subagent layouts, reviewer personas, or implementation tactics
- Background reconciliation, notifications, dashboards, or trend monitoring
- Automatic promotion of reusable knowledge
- A custom template engine or centrally hosted CI workflow before repeated use demonstrates that boundary

## References

- [Reference workflow findings](research/reference-workflows.md)
- [ADR 0001: Keep the first version thin](../adr/0001-thin-first-version.md)
- [ADR 0003: Keep CI templates in user knowledge](../adr/0003-keep-ci-templates-in-user-knowledge.md)
- [ADR 0004: Use GitHub Issues for project tasks](../adr/0004-use-github-issues-for-project-tasks.md)
- The [archived autodev repository](https://github.com/syshin0116/autodev-archive) is historical evidence, not an active design contract.

## Open questions

None that can change the first-version scope or verification.
