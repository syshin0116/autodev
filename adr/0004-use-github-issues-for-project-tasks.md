---
status: accepted
date: 2026-08-10
---

# ADR 0004: Use GitHub Issues for project tasks

## Context

The local Task Graph proved the interview, approval, and execution boundary without adding a service. Ongoing Autodev work can grow beyond a planning file that remains easy to review and edit. This repository now provides the real second Task System of Record required by the upgrade trigger in [ADR 0001](0001-thin-first-version.md).

GitHub Issues provides native sub-issues, blocking relationships, search, and APIs. A parent can have up to 100 direct sub-issues and GitHub supports eight hierarchy levels, so large plans can be partitioned without loading every task at once. Keeping the same task definitions in both Issues and `tasks.yaml` would reintroduce the drift this project is intended to prevent.

## Decision

After one approved migration task, `syshin0116/autodev` will use GitHub Issues as its sole Task System of Record.

This supersedes only ADR 0001's local Task System of Record choice for this repository after migration. Its remaining boundaries stay active.

One configured root issue is a non-executable plan container included in the approval projection. Its recursive sub-issues define task membership, hierarchy, and deterministic traversal order. Every task must belong to `syshin0116/autodev`. Native issue dependencies define blocking edges and both endpoints must be task members. Pull requests, cross-repository sub-issues, and external dependency endpoints make the graph invalid. Issue identity and number remain stable references within this repository.

Each issue body contains the outcome, local planning references, and verification checks. GitHub owns those task definitions after migration. No local file mirrors them as another writable source.

Approval binds the Project Overview bytes and a deterministic projection of:

- repository and root issue
- stable issue identity and number
- title and body
- parent membership and sibling order
- native blocking edges

The projection excludes open or closed state, assignees, comments, and labels. Those fields may change during execution without changing the approved plan, and none of them substitutes for verified evidence. Any included field or graph edge changing after approval invalidates the digest.

Validation exhausts pagination and fails closed when the complete graph or dependencies cannot be read, including authentication, authorization, rate-limit, and network failures. Execution evidence refers to the issue and approval digest without copying the task definition.

The current local Task Graph remains authoritative only while implementing and verifying this support. A later planning revision creates the Issue Graph, presents its complete planning projection for approval, switches configuration, and then removes `tasks.yaml`. The two sources are never simultaneously executable.

No generic Task adapter framework is introduced. Another provider is added only when a real project selects it.

## Considered options

### Keep one growing local Task Graph

This preserves offline validation but makes large plans harder to review, filter, and update safely.

### Mirror GitHub Issues into a local Task Graph

This keeps the current planning revision validator simple but creates two writable representations and requires reconciliation.

### Use GitHub Issues as the source and hash a planning projection

This is the selected option. It uses the repository's native task system while retaining exact-revision approval.

## Consequences

- This project's task planning and execution require readable GitHub state.
- Ordinary progress metadata does not invalidate approval.
- Task definition or dependency edits require a new approval.
- Offline snapshots may be retained as evidence or caches, but never as a second task source.
- The Skill and planning revision validator remain provider-specific until another real task system justifies a shared boundary.

## References

- [GitHub: About issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/learning-about-issues/about-issues)
- [GitHub: Adding sub-issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues)
- [GitHub: REST API endpoints for sub-issues](https://docs.github.com/en/rest/issues/sub-issues)
- [GitHub: Creating issue dependencies](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-issue-dependencies)
- [GitHub: REST API endpoints for issue dependencies](https://docs.github.com/en/rest/issues/issue-dependencies)
