---
id: autodev-rebuild
---

# Project Overview

## Background

Autodev is being rebuilt as the planning boundary between a rough request and an existing Agent Host. It must conduct a serious, decision-focused interview, selectively reuse prior Markdown knowledge, reduce the result to concise planning artifacts, and wait for content-bound approval before handoff.

- Adopted from the earlier implementation: defer orchestration layers, dashboards, and adapters until use proves they are needed. (Source: [previous-autodev-retrospective.md](../../knowledge/previous-autodev-retrospective.md).)
- Adopted from the earlier implementation: keep standing instructions minimal, bind approval to the reviewed bytes, and retain decision context only when it helps future reuse. (Source: [previous-autodev-retrospective.md](../../knowledge/previous-autodev-retrospective.md).)
- Not carried over: legacy formats, architecture, and historical decisions not restated in this Overview. The retrospective is evidence, not authority. (Source: [previous-autodev-retrospective.md](../../knowledge/previous-autodev-retrospective.md).)

## Goal

Provide a host-neutral way to turn a rough software or non-software request into one decision-complete Project Overview and one dependency-aware local Task Graph, then create a tamper-evident approval record that an existing Agent Host can validate before execution.

## Scope

- Interview only across unresolved choices that can change the goal, success criteria, scope, constraints, dependencies, risks, or verification.
- Search only user-selected, read-only Markdown roots on demand; read plausible matches selectively and cite records that affect the plan.
- Keep one visible Project Overview as canonical planning state and one local YAML Task Graph derived from it.
- Keep project configuration and exact-byte approval in `.autodev`, with deterministic validation before handoff.
- Support software and non-software projects through the same host-neutral contract.
- Provide the minimal templates, guidance, checks, and examples needed for another Agent Host to use the contract correctly.

## Out of scope

- Executing project tasks or supplying a task runner, daemon, scheduler, or orchestration service.
- Dashboards, knowledge indexes, vector databases, or writes to selected knowledge roots.
- Vendor-specific Agent Host adapters, prompts, or execution tactics.
- Compatibility or migration support for the earlier Autodev implementation.
- Interview transcripts, duplicate briefs, or planning state outside the Overview, Task Graph, configuration, and approval record.

## Decisions

- The first release ends at an approved handoff. The existing Agent Host owns execution.
- A project has one configured Overview and one configured YAML Task Graph. Configuration and approval records are control files, not competing planning documents.
- Knowledge roots remain user-owned, explicitly selected, read-only, and searched only when request terms make a record plausible. Session-only roots and sensitive paths are not persisted without an explicit request.
- Prior records provide context only. The Overview states what was adopted and what does not carry over, with traceable citations that do not expose private absolute paths.
- The Task Graph contains verifiable outcomes, dependencies, project-local planning references, and concrete checks, while leaving execution tactics to the Agent Host.
- `.autodev/approval.yaml` is the sole approval authority. It remains pending until the complete revision is shown and the user unambiguously approves that exact content.
- Approval records the approver, approval time, and SHA-256 digest of the exact Overview and Task Graph bytes. Any later byte change invalidates approval and requires review of a new revision.
- Validation must reject unresolved questions, malformed or cyclic task graphs, missing project-local references, incomplete approval metadata, and digest mismatches.
- The rebuild is a clean break. No speculative compatibility layer is included.

## Success criteria

- A rough request produces only material, grouped follow-up questions; once answered, the Overview declares `None.` under Open questions without an interview transcript or second brief.
- In both a software and a non-software planning scenario, the resulting Overview states a testable goal, boundaries, decisions, and success criteria, and the Task Graph contains unique tasks with valid dependencies, local references, outcomes, and non-empty checks.
- A knowledge-boundary check demonstrates that only selected Markdown roots are searched, selected files are unchanged, unselected directories are not scanned, and every adopted record is cited safely beside the decision it affected.
- Before approval, the full Overview and Task Graph are shown together and the approval record remains pending. Silence, the initial request, and earlier acknowledgments do not approve the revision.
- After explicit approval, the record identifies the approver and time and covers exactly the configured Overview and Task Graph with matching SHA-256 digests; changing either file makes validation fail.
- Project validation deterministically rejects every unresolved, structurally invalid, cyclic, missing-reference, pending, incomplete, or byte-mismatched fixture and accepts a complete approved fixture.
- The documented handoff lets an Agent Host locate and validate the artifacts without a vendor-specific extension, and Autodev does not execute any Task Graph item.
- The shipped first release contains no task runner, daemon, dashboard, vendor adapter, knowledge index, vector database, or legacy migration layer.

## Open questions

None.
