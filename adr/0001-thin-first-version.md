---
status: accepted
date: 2026-08-09
---

# ADR 0001: Keep the first version thin

## Context

Autodev needs to preserve a knowledge-aware interview, a concise planning contract, explicit approval, execution evidence, and reusable learning without becoming another Agent Host or specification framework.

The [reference review](../docs/research/reference-workflows.md) found useful parts in existing projects, but no single implementation covers this boundary. User reports repeatedly identify interview fatigue, duplicated documents, planning drift, noisy memory capture, and task infrastructure that becomes a product of its own.

The archived autodev system contains additional orchestration and governance machinery. Its decisions remain historical evidence unless this repository adopts them explicitly.

## Decision

Build the first version as one portable Agent Skill with a small file contract and a standard-library approval check.

- The Skill produces one visible Project Overview and one Task Graph.
- The first Task System of Record is a local file.
- `.autodev/` contains only machine configuration and state.
- User knowledge remains in configured, user-owned Markdown roots.
- Approval binds the exact Overview and Task Graph revision.
- Execution and verification use the existing Agent Host.
- Execution evidence is separate from the approved planning revision.
- Reusable learnings are sourced candidates, never automatic knowledge writes.

No custom runtime, server, graph database, retrieval engine, Task database, background reconciliation process, or Host-specific execution strategy is part of the first version.

## Considered options

### Adopt a complete specification framework

This supplies more workflow immediately but also imports overlapping artifacts, code-project assumptions, and standing instructions that autodev does not need.

### Compose several existing systems as runtime dependencies

This provides specialized features but makes the first usable path depend on their storage, lifecycle, and compatibility choices.

### Define a thin Skill and file boundary

This is the selected option. It tests the distinctive value of autodev before adding infrastructure: whether personal knowledge and focused interviewing produce a plan worth approving and executing.

## Consequences

- The first milestone can be dogfooded without a service or new dependency.
- Users retain readable project and knowledge artifacts.
- The first version has local-file limitations and no automatic notification for pending approvals.
- Agent Hosts may behave differently. Only observed compatibility differences justify an adapter or mapping.
- A later integration must preserve the same artifact and approval semantics.

## Upgrade triggers

- Add a Task adapter only after a second System of Record is used in a real project.
- Add a derived search index only after Markdown search fails a recorded retrieval case.
- Add runtime orchestration only after an Agent Host cannot preserve the approved handoff and evidence boundary.
- Add approval notifications only after durable pending state and normal project re-entry fail to surface approvals reliably.
