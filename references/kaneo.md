# Kaneo

Use this reference only when `task_source.type` is `kaneo`.

## Project mapping

Require one project-scoped Kaneo MCP server and one exact mapping:

```yaml
task_source:
  type: kaneo
  server: https://cloud.kaneo.app/api/mcp
  workspace_id: WORKSPACE_ID
  project_id: PROJECT_ID
```

The server is the configured MCP endpoint, not an API token. Verify the workspace and project with `list_workspaces`, `list_projects`, and `get_project`. Never infer a project from a similar name when an ID is absent or ambiguous. Do not install, authenticate, or reconfigure Kaneo unless the user separately asks.

## Task shape

Each task description contains these sections:

```markdown
## Outcome

The state this task must create.

## Planning references

- docs/product-requirements.md#relevant-section

## Verification

- A runnable or human-verifiable check.
```

Use `blocks` for execution dependencies. Keep `subtask` and `related` relations only when they express a real relationship. Status, priority, assignee, dates, labels, and comments are operational metadata rather than planning content.

## Fresh projection

Before review and every validation:

1. Verify the configured project identity.
2. Call `list_tasks` with only `projectId`, without `page` or `limit`. Require `pagination.page: 1`, `pagination.totalPages: 1`, and `pagination.pageSize` equal to `pagination.total`. Flatten tasks from every column plus `plannedTasks` and `archivedTasks`.
3. Read `get_task_relations` for every task. Duplicate relation records are allowed because each endpoint may return the same relation.
4. Build one JSON input with this exact shape:

```json
{
  "server": "https://cloud.kaneo.app/api/mcp",
  "workspace_id": "WORKSPACE_ID",
  "project_id": "PROJECT_ID",
  "tasks": [
    {
      "id": "TASK_ID",
      "number": 1,
      "project_id": "PROJECT_ID",
      "title": "Task title",
      "description": "## Outcome\n..."
    }
  ],
  "relations": [
    {
      "source_task_id": "BLOCKER_ID",
      "target_task_id": "BLOCKED_ID",
      "relation_type": "blocks"
    }
  ]
}
```

Write this input only to a temporary file. Do not add a Kaneo mirror to the project. The validator sorts tasks and relations, removes duplicate relations, verifies project membership, validates task sections and local references, rejects dependency cycles, and prints the canonical projection and digest.

Inspect the current projection:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --print-kaneo-task-projection --root <project-root> --input <temporary-json>
```

Validate a fresh projection before task work:

```sh
cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- \
  --validate-kaneo-task-projection --root <project-root> --input <temporary-json>
```

If a fresh complete read is unavailable, stop. Never reuse an earlier projection as current state.

## Writes

Selecting Kaneo identifies the Task Source but does not authorize a mutation. Confirm the mapped project and proposed change before creating, editing, deleting, or relating tasks. During execution, update task status only when the user's request or project instructions authorize board progress updates. Mark a task complete only after its verification and Autodev evidence succeed.
