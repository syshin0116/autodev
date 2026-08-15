---
name: autodev
description: Plan, approve, and execute a software or non-software project using selected Markdown knowledge. Use when the user wants a focused interview, concise Project Overview, dependency-aware Task Graph, content-bound approval, evidence-backed execution, or reusable learning candidates.
---

# Autodev

Plan and approve on the first pass. Execute only after project approval and task authorization, on a later request or trusted event.

## Route the request

Resolve phase-guide, template, and capability-document paths relative to this Skill root. Resolve configured project paths as their phase guide specifies.

Read the selected phase guide completely before acting:

- For project setup, knowledge search, interviewing, planning, revision, or approval, read [Planning](references/planning.md).
- For a trusted task event or an explicit request to execute, reverify, or rerun a task, read [Execution](references/execution.md).
- After verified execution, or when reviewing learning candidates at project close, read [Learning](references/learning.md).

For a status or continuation request, inspect the configured planning, approval, and evidence state first, then load only the relevant phase. If `.autodev/config.yaml` is absent, only Planning may initialize the project contract.

## Preserve cross-phase boundaries

- Treat `.autodev/approval.yaml` as the sole project-revision approval authority. Approval is bound to the exact configured planning revision. In rootless GitHub mode, separately require a trusted authorization for each exact Issue snapshot.
- Never hide a planning change by refreshing its digest. Return approval to pending and request approval for the changed revision.
- Fail closed when Planning Revision Validation fails. Run it immediately before task work and again after task checks, before writing evidence.
- Approval never bypasses the Agent Host's permission and safety boundaries.
- Keep selected knowledge roots read-only. Reusable learnings remain pending candidates until explicitly reviewed.
