# Project Validation Capability

## Purpose

Prevent execution against an unresolved or unapproved planning revision.

## Inputs

- A project root
- Machine configuration that locates the Project Overview and Task Graph
- YAML planning and approval records

Configured paths must be project-relative and resolve inside the project root. The Approval Record is `.autodev/approval.yaml`.

## Valid project

A project is valid when all of the following hold:

- The Overview status is `approved`.
- Its `Open questions` section begins with `None` after comments and whitespace are removed.
- The Task Graph status is `approved` and contains at least one task.
- Task IDs are non-empty and unique.
- Dependencies name existing tasks and contain no cycle.
- Every task has a title, at least one planning reference, and at least one non-empty verification check.
- Referenced planning files exist inside the project root.
- The Approval Record is approved, identifies the approver and time, and covers exactly the configured Overview and Task Graph.
- Each recorded SHA-256 digest matches the current file bytes.

The `Open questions` check validates the declared planning state. Deciding whether an omitted question was material remains part of the interview and human review.

## Result

The capability returns success only when every check passes. Failure reports deterministic errors and blocks execution without modifying project files.

## Exclusions

- Judging whether the plan is good
- Selecting implementation tactics
- Updating approval after a file changes
- Validating execution evidence
