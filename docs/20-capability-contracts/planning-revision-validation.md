# Planning Revision Validation Capability

- [Purpose](#purpose)
- [Inputs](#inputs)
- [Valid planning revision](#valid-planning-revision)
- [Commands](#commands)
- [Result](#result)

## Purpose

Block execution when the configured Project Overview or project planning contract is unresolved, structurally invalid, unreadable, or different from the approved revision.

It validates project approval and the control state of a requested rootless task snapshot. It does not persist task authorization or judge plan quality, delivery readiness, or evidence.

## Inputs

- A project root
- `.autodev/config.yaml`
- the configured Project Overview
- exactly one task source
- `.autodev/approval.yaml`

The Overview and local planning references must resolve inside the project root.

### Local task source

The existing `task_graph: tasks.yaml` form remains supported. The explicit equivalent is:

```yaml
task_source:
  type: local_file
  path: tasks.yaml
```

Local approval uses the existing `files` mapping and binds the exact Overview and Task Graph bytes. An optional `required_planning_transition.after_task` must name a task in that graph.

### Rooted GitHub Issues task source

Existing rooted projects remain supported:

```yaml
task_source:
  type: github_issues
  repository: OWNER/REPO
  root_issue: 123
```

The root issue is a non-executable container. Its recursive sub-issues are tasks. Native blocking relationships are dependencies. Approval records the Overview digest and deterministic Issue Graph projection digest:

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

### Rootless GitHub Issues task source

New GitHub projects may omit a root:

```yaml
task_source:
  type: github_issues
  repository: OWNER/REPO
```

Approval binds the complete project-revision projection, not a collection of Issues. The projection digest covers the Overview identity below and the semantic project configuration:

```yaml
planning_revision:
  project_overview:
    path: docs/project-overview.md
    sha256: <digest>
  project:
    sha256: <projection-digest>
```

The projection binds:

- Task Source repository name and immutable identity
- ready and refusal labels, plus authorizer and cancel roles
- delivery base branch, protected paths, required checks, and review rules
- registered tool names, project purposes, interfaces, and permission envelopes
- Knowledge source aliases and visibility, treating omitted visibility as private
- candidate carrier and writable Knowledge target identities and visibility
- merge mode and correction budget
- engine policy provider, data-use boundary, and cost class

It excludes credential values, machine-specific Knowledge paths, installation and authentication state, and the selected engine and version allowed by that policy. The selected engine and version remain operational evidence.

Each task is later authorized against one canonical Issue snapshot. That snapshot binds repository, immutable Issue identity, number, title, raw body, direct dependency identities, and the approved project-revision digest. The adapter separately supplies integrity-filtered Agent input and records its digest beside the raw snapshot digest.

Authorization and episode transitions are deterministic state decisions. The delivery adapter must serialize each Issue, persist the returned record before side effects, and supply that record on the next decision. This capability does not provide a lock or writable state carrier.

## Valid planning revision

Every source requires:

- `Open questions` begins with `None` after comments and whitespace are removed.
- The Approval Record is approved and identifies the approver and time.
- The current source matches the recorded approval digest.

A local Task Graph also requires non-empty unique task IDs, titles, outcomes, verification checks, valid project-relative references, known dependencies, and no dependency cycle.

A rooted GitHub Issue Graph also requires:

- complete paginated reads of the root, recursive sub-issues, and both dependency directions
- stable issue identity, hierarchy, sibling order, title, and raw body in the projection
- `Outcome`, `Planning references`, and `Verification` body sections with plain bullets
- every task and dependency endpoint in the configured repository and root membership
- no pull request in task membership or dependency endpoints
- known dependencies and no dependency cycle

A rootless GitHub project also requires every projected policy group, a complete valid semantic projection, and an approval digest for it. It does not enumerate Issues during project-revision validation.

Every GitHub task snapshot requires the three task body sections, a same-repository identity for the task and every dependency endpoint, and no pull request. A failure to read the exact task or its complete direct dependency pages fails closed. Changes to unrelated Issues do not affect its authorization.

GitHub project projections exclude current Issue state, applied-label membership, assignees, and comments. Those fields may change without changing project approval and never satisfy verification.

## Commands

Validate the current approved revision:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- <project-root>
```

Print the current rootless project-revision projection and SHA-256 before approval:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-project-revision <project-root>
```

Print the current rooted GitHub Issue Graph projection and SHA-256 before approval:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-task-projection <project-root>
```

Validate approval and print the exact rooted GitHub snapshot to use for execution:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-validated-task-projection <project-root>
```

### Rootless delivery decisions

These commands are for a trusted adapter job, never for the Agent. Each reads its inputs from files, prints one JSON decision, and writes nothing. The adapter owns per-issue serialization and persistence.

Validate approval and print the raw snapshot for one authorized issue. It requires the approved ready label, so an unlabeled issue fails closed:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --print-task-snapshot --root <project-root> --issue <number>
```

Decide one ready event. `--event` is a `ReadyEvent`, `--agent-input` is the integrity-filtered bytes the Agent will receive, and `--prior` is the durable authorization record list for that issue, omitted for a first authorization:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --authorize --root <project-root> --issue <number> \
  --event <ready-event.json> --agent-input <agent-input> [--prior <authorizations.json>]
```

Decide one episode transition. `--current` is the authorization record returned by the previous decision:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --transition --root <project-root> --event <episode-event.json> --current <authorization.json>
```

Complete a merged episode. `--evidence` is the verified evidence for that exact episode, and `--merged-into` must be the approved base branch:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --complete-merge --root <project-root> --current <authorization.json> \
  --evidence <evidence.json> --merged-into <branch>
```

The printed record is the next durable state. Persist it before any side effect, then supply it as `--prior` or `--current` on the following decision. Replaying the same event identity returns a no-op rather than a second episode.

## Result

Success means only that the declared project plan is closed, structurally valid, readable, and identical to the approval record. Rooted execution uses the Issue Graph projection returned by that validation call. A trusted Host-side adapter uses the Rust library boundary to retain a rootless raw task snapshot outside Agent input, then passes only integrity-filtered input and the two digests onward. Any incomplete GitHub read, API error, malformed response, or digest mismatch fails closed without modifying project files.
