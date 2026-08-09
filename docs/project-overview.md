---
id: autodev
status: approved
approval: user-approved-in-chat-2026-08-09
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

## Deliverables

- One canonical Project Overview
- An approved Task Graph with dependencies and completion checks
- Execution artifacts and verification evidence
- Reviewable learning candidates

## Flow

```text
knowledge lookup -> interview -> overview -> tasks -> approve -> host execution -> verify -> propose learnings
```

The first milestone ends at the approved handoff. Execution and learning reuse the Agent Host instead of introducing an autodev runtime.

## Decisions

- Package autodev as one portable Agent Skill. The Agent Host owns execution strategy.
- Keep user knowledge outside the distributed autodev repository in user-owned Markdown roots.
- Keep the Project Overview canonical. Add a Decision Record only when the alternatives and rationale will matter later.
- Store user-facing project artifacts at visible, configured paths. Reserve `.autodev/` for machine-readable configuration and state.
- Use one selected Task System of Record. The first milestone uses a local file and adds another integration only after a real project needs it.
- Treat the approved Overview and Task Graph as an immutable planning revision. Store execution status and evidence separately so normal progress does not invalidate approval.
- Bind approval to the exact planning revision. Reopen approval only when the goal, scope, completion criteria, or task dependencies materially change.
- Treat approved decisions as accepted within the project only. A reusable lesson still enters the writable Knowledge Root as a candidate.
- Propose only sourced learning candidates with context and applicability. Check accepted, pending, deferred, and dismissed records before proposing another.
- Review new candidates in one batch at project close. A deferred candidate remains searchable and may resurface during later relevant work without blocking closure.
- Present the minimum decision-relevant summary first, then link to Decision Records and raw evidence.

These boundaries are accepted in [ADR 0001](../adr/0001-thin-first-version.md).

## Success criteria

- A free-form idea or supplied opportunity can reach an Overview with no unresolved question that could change scope, dependencies, or verification.
- Prior knowledge informs the interview and is cited without silently becoming binding.
- The Overview contains only information that changes a decision, action, constraint, or verification result.
- Every task links to the relevant Overview section or Decision Record and has a runnable or human-verifiable completion check.
- Execution is blocked when approval is missing or either approved planning artifact changes.
- Execution evidence can accumulate without changing the approved planning revision.
- A completed run can produce a sourced learning candidate without promoting it automatically.
- The same Skill passes the approval boundary in at least two Agent Hosts without changing the core package.
- One non-software fixture reaches approval without code-specific assumptions.

## Non-goals for the first version

- Automatic opportunity discovery
- A custom model runtime, server, chat UI, or Task database
- A hosted knowledge service, graph database, embedding pipeline, or custom retrieval engine
- Task integrations beyond the initial local System of Record
- Prescribed subagent layouts, reviewer personas, or implementation tactics
- Background reconciliation, notifications, dashboards, or trend monitoring
- Automatic promotion of reusable knowledge

## References

- [Reference workflow findings](research/reference-workflows.md)
- [ADR 0001: Keep the first version thin](../adr/0001-thin-first-version.md)
- The [archived autodev repository](https://github.com/syshin0116/autodev-archive) is historical evidence, not an active design contract.

## Open questions

None that can change the first-version scope or verification.
