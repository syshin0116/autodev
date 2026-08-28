# Planning Revision Validation Capability

- [Purpose](#purpose)
- [Inputs](#inputs)
- [Valid planning revision](#valid-planning-revision)
- [Commands](#commands)
- [Result](#result)

## Purpose

Block execution when the configured Project Overview or project planning contract is unresolved, structurally invalid, unreadable, untracked, or different from committed Git state.

It validates committed local planning state and current external task projections. It also validates the control state of a requested rootless task snapshot. It does not persist task authorization or judge plan quality, delivery readiness, or evidence.

## Inputs

- A project root
- `.autodev/config.yaml`
- the configured Project Overview
- exactly one task source

The configuration, Overview, and local Task Graph must resolve inside the project root, be tracked by Git, and match `HEAD` before execution. External Task Sources are read fresh instead of mirrored into a local approval record.

### Local task source

The existing `task_graph: tasks.yaml` form remains supported. The explicit equivalent is:

```yaml
task_source:
  type: local_file
  path: tasks.yaml
```

An optional `required_planning_transition.after_task` must name a task in that graph.

### Kaneo task source

Kaneo selects one existing Agent Host MCP connection and one exact project:

```yaml
task_source:
  type: kaneo
  server: https://cloud.kaneo.app/api/mcp
  workspace_id: WORKSPACE_ID
  project_id: PROJECT_ID
```

The Host supplies a fresh complete JSON projection input from Kaneo. The validator binds task ID, number, project, title, description, and `blocks`, `subtask`, and `related` relations. It excludes status, priority, assignee, dates, comments, and labels. The input is temporary and never becomes a second Task System of Record. Its digest is an internal integrity output and is not copied into project configuration.

### Rooted GitHub Issues task source

Existing rooted projects remain supported:

```yaml
task_source:
  type: github_issues
  repository: OWNER/REPO
  root_issue: 123
```

The root issue is a non-executable container. Its recursive sub-issues are tasks. Native blocking relationships are dependencies. Validation reads the current complete Issue Graph and returns its deterministic projection.

### Rootless GitHub Issues task source

New GitHub projects may omit a root:

```yaml
task_source:
  type: github_issues
  repository: OWNER/REPO
```

The internal project-revision projection covers the Overview identity and semantic project configuration below, not a collection of Issues. The trusted delivery adapter uses its digest to bind exact task authorization and episode state. Users do not maintain that digest.

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

Each task is later authorized against one canonical Issue snapshot. That snapshot binds repository, immutable Issue identity, number, title, raw body, direct dependency identities, and the project-revision digest. The adapter separately supplies integrity-filtered Agent input and records its digest beside the raw snapshot digest.

Authorization and episode transitions are deterministic state decisions. The delivery adapter must serialize each Issue, persist the returned record before side effects, and supply that record on the next decision. This capability does not provide a lock or writable state carrier.

## Valid planning revision

Every source requires:

- `Open questions` begins with `None` after comments and whitespace are removed.
- The configuration and Overview are tracked and match committed Git state.
- Every external source read is current and complete.

A local Task Graph also requires non-empty unique task IDs, titles, outcomes, verification checks, valid project-relative references, known dependencies, and no dependency cycle.

A Kaneo Task Graph also requires a complete project read, non-empty unique task IDs and numbers, project membership for every task and relation endpoint, the three task-description sections, valid project-relative references, supported relation types, and no `blocks` cycle. Response order and duplicate relation reads do not change its digest.

A rooted GitHub Issue Graph also requires:

- complete paginated reads of the root, recursive sub-issues, and both dependency directions
- stable issue identity, hierarchy, sibling order, title, and raw body in the projection
- `Outcome`, `Planning references`, and `Verification` body sections with plain bullets
- every task and dependency endpoint in the configured repository and root membership
- no pull request in task membership or dependency endpoints
- known dependencies and no dependency cycle

A rootless GitHub project also requires every projected policy group and a complete valid semantic projection. It does not enumerate Issues during project-revision validation.

Every GitHub task snapshot requires the three task body sections, a same-repository identity for the task and every dependency endpoint, and no pull request. A failure to read the exact task or its complete direct dependency pages fails closed. Changes to unrelated Issues do not affect its authorization.

GitHub project projections exclude current Issue state, applied-label membership, assignees, and comments. Those fields may change without changing the project revision and never satisfy verification.

## Commands

Validate the current committed planning state:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- <project-root>
```

Print the current rootless project-revision projection and internal SHA-256 for adapter inspection:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-project-revision <project-root>
```

Print the current rooted GitHub Issue Graph projection and internal SHA-256:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-task-projection <project-root>
```

Validate committed local planning state and print the current rooted GitHub snapshot to use for execution:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- --print-validated-task-projection <project-root>
```

Print a canonical Kaneo Task Graph projection and SHA-256 from a fresh MCP read:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --print-kaneo-task-projection --root <project-root> --input <temporary-json>
```

Validate a fresh Kaneo projection before and after task work:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --validate-kaneo-task-projection --root <project-root> --input <temporary-json>
```

### Rootless delivery decisions

These commands are for a trusted adapter job, never for the Agent. Each reads its inputs from files, prints one JSON decision, and writes nothing. The adapter owns per-issue serialization and persistence.

Validate committed project state and print the raw snapshot for one authorized issue. It requires the configured ready label, so an unlabeled issue fails closed:

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

Complete a merged episode. `--evidence` is the verified evidence for that exact episode, and `--merged-into` must be the configured base branch:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --complete-merge --root <project-root> --current <authorization.json> \
  --evidence <evidence.json> --merged-into <branch>
```

Check whether one task's dependencies are complete. `--statuses` is one status per declared dependency:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --dependencies-ready --root <project-root> --issue <number> --statuses <statuses.json>
```

The printed record is the next durable state. Persist it before any side effect, then supply it as `--prior` or `--current` on the following decision. Replaying the same event identity returns a no-op rather than a second episode.

## Result

Success means only that the declared project plan is closed, structurally valid, readable, and committed, and that the external read is complete when applicable. Rooted GitHub and Kaneo execution use the fresh Task Graph projection returned by validation. A trusted Host-side adapter uses the Rust library boundary to retain a rootless GitHub raw task snapshot outside Agent input, then passes only integrity-filtered input and internal digests onward. Any incomplete external read, API error, malformed response, uncommitted planning change, or internal digest mismatch fails closed without modifying project files.
