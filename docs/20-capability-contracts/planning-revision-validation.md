# Planning Revision Validation Capability

## Purpose

Block execution when the configured Project Overview or Task Graph is unresolved, structurally invalid, unreadable, or different from the approved revision.

It does not judge plan quality, execution readiness, permissions, or task evidence.

## Inputs

- A project root
- `.autodev/config.yaml`
- The configured Project Overview
- Exactly one task source
- `.autodev/approval.yaml`

The Overview and local planning references must resolve inside the project root.

### Local task source

The existing `task_graph: tasks.yaml` form remains supported. The explicit equivalent is:

```yaml
task_source:
  type: local_file
  path: tasks.yaml
```

Local approval uses the existing `files` mapping and binds the exact Overview and Task Graph bytes.

### GitHub Issues task source

```yaml
task_source:
  type: github_issues
  repository: OWNER/REPO
  root_issue: 123
```

The root issue is a non-executable container. Its recursive sub-issues are tasks. Native blocking relationships are dependencies.

GitHub approval records the Overview digest and the deterministic Issue Graph projection digest:

```yaml
planning_revision:
  project_overview:
    path: docs/project-overview.md
    sha256: <digest>
  task_source:
    type: github_issues
    repository: OWNER/REPO
    root_issue: 123
    sha256: <projection-digest>
```

## Valid planning revision

Every source requires:

- `Open questions` begins with `None` after comments and whitespace are removed.
- The Approval Record is approved and identifies the approver and time.
- Every task has a title, local planning reference, outcome, and verification check.
- Dependencies name tasks in the same graph and contain no cycle.
- The current source matches the recorded approval digest.

A local Task Graph also requires non-empty unique task IDs and valid project-relative references.

A GitHub Issue Graph also requires:

- complete paginated reads of the root, recursive sub-issues, and both dependency directions
- stable issue identity, hierarchy, sibling order, title, and raw body in the projection
- `Outcome`, `Planning references`, and `Verification` body sections with plain bullets
- every task and dependency endpoint in the configured repository and root membership
- no pull request in task membership or dependency endpoints

The GitHub projection excludes state, labels, assignees, and comments. Those fields may change without changing approval and never satisfy verification.

## Commands

Validate the current approved revision:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- <project-root>
```

Print the current GitHub planning projection and SHA-256 before approval:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-task-projection <project-root>
```

Validate approval and print the exact GitHub snapshot to use for execution:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-validated-task-projection <project-root>
```

## Result

Success means only that the declared plan is closed, structurally valid, readable, and identical to the approval record. GitHub execution uses the projection returned by that validation call, not a second read. Any incomplete GitHub read, API error, malformed response, or digest mismatch fails closed without modifying project files.
