---
status: accepted
date: 2026-08-26
---

# ADR 0008: Use Git instead of approval hashes

## Context

Autodev stored approver metadata and planning-file digests in `.autodev/approval.yaml`. Every accepted documentation change required a second file update. CI then failed when the committed plan and manually maintained hash record drifted, even though Git already preserved the reviewed content and its history.

External Task Sources made the duplicate record more costly. GitHub Issues and Kaneo had to be read again before execution anyway, so a saved projection digest was another stale copy rather than the current source.

## Decision

- Use the committed `.autodev/config.yaml`, Project Overview, and local Task Graph as durable local planning state.
- Require those files to be tracked by Git and equal to `HEAD` during Planning Revision Validation.
- Read GitHub Issues and Kaneo fresh before execution. Do not mirror their task state into an approval file.
- Keep explicit user review before recording planning mutations. Use normal commit and pull-request review when the repository requires it.
- Keep internal project, task, Agent-input, authorization-generation, and evidence digests where they prevent stale or misrouted execution. Users do not copy or refresh them.
- Keep the trusted ready-label event for exact rootless GitHub task authorization. A committed project revision does not authorize an Issue.

This supersedes earlier ADR requirements for a manually maintained Approval Record. Their other planning, execution, and evidence boundaries remain active.

## Consequences

- Planning changes no longer require synchronized hash edits.
- CI validates the committed plan directly.
- A local commit is sufficient for conversational execution. Repositories that require human review enforce it through their normal pull-request and branch rules.
- An uncommitted planning change blocks execution.
- External task edits become current input on the next fresh read. Rootless GitHub still rejects stale task authorization through its internal snapshot digest.
