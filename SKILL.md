---
name: autodev
description: Plan, review, and execute a software or non-software project using selected Markdown knowledge. Use when the user wants a focused interview, concise Project Overview, dependency-aware Task Graph, evidence-backed execution, or reusable learning candidates.
---

# Autodev

Plan and review on the first pass. Execute only from committed planning state, on a later request or trusted event.

## Route the request

Resolve phase-guide, template, and capability-document paths relative to this Skill root. Resolve configured project paths as their phase guide specifies.

Read the selected phase guide completely before acting:

- For project setup, knowledge search, interviewing, planning, revision, or review, read [Planning](references/planning.md).
- For a trusted task event or an explicit request to execute, reverify, or rerun a task, read [Execution](references/execution.md).
- After verified execution, or when reviewing learning candidates at project close, read [Learning](references/learning.md).

When the selected Task Source is Kaneo, also read [Kaneo](references/kaneo.md). Use the Agent Host's existing Kaneo MCP connection. Autodev does not install the server, authenticate it, or store its credentials.

For a status or continuation request, inspect the configured planning, Git, external task source, and evidence state first, then load only the relevant phase. If `.autodev/config.yaml` is absent, only Planning may establish it through first-use discovery and a focused setup interview. Do not require a separate init command.

## Preserve cross-phase boundaries

- Treat committed configured planning files as the durable local project state. Do not maintain a separate approval file or ask the user to copy hashes.
- Read GitHub Issues or Kaneo again immediately before execution. In rootless GitHub mode, require a trusted authorization for each exact Issue snapshot.
- Fail closed when Planning Revision Validation fails. Run it immediately before task work and again after task checks, before writing evidence.
- Planning review or task authorization never bypasses the Agent Host's permission and safety boundaries.
- Keep selected knowledge roots read-only. Reusable learnings remain pending candidates until explicitly reviewed.
