# Execution

Use this phase only on a later request to execute, reverify, or rerun an approved task.

1. Run Planning Revision Validation immediately before any task work, using the entry point in `docs/10-runtime-mapping.md` relative to the Autodev Skill root. On failure, stop without creating an output, evidence record, or learning candidate. After it succeeds, read each local approval-bound planning file once, hash those exact bytes against the Approval Record, and retain the same-byte snapshots for this task. For GitHub, also use the validated-projection entry point and retain its returned snapshot. Stop on any mismatch, and do not reread an approval-bound file or the Issue Graph for task selection or execution.
2. Read the current approval revision from `planning_revision`, or from the legacy `files` mapping for a local task source. Read the validated local Task Graph or the GitHub snapshot returned by step 1. The GitHub root issue is not a task, and Issue state does not prove completion. Use the local task ID as its evidence key. Use `OWNER/REPO#NUMBER` for a GitHub task. Scan Markdown files under `evidence/` instead of deriving a path from either unchecked key. A task or dependency is complete when one record has that exact YAML string in `task`, has `status: verified`, and repeats the current approval revision as `planning_revision`. Only when no current record exists, treat verified evidence for another planning revision as stale and exclude that task from automatic selection.
3. A task is ready when it has no current or stale verified evidence and every dependency does. If the user requested a task, run only that task when ready, handle it under step 4 when stale, or stop and report any other state. Only when no task was requested, use the first ready task in the validated task-source order.
4. If the user explicitly requests a stale task, first require every dependency to be complete for the current revision. Then show the revision conflict and ask whether to reverify or rerun unless the request already says which. For reverify, skip task work and continue with the current checks in step 7. Rerun only after an explicit request and normal permission checks.
5. Read every planning reference selected by the task, using a retained snapshot when it targets an approval-bound file. Then let the Agent Host choose tactics from the task outcome, references, and verification checks. Approval of the plan does not bypass normal permission or safety boundaries for destructive, sensitive, costly, or external actions.
6. If execution exposes a material change to the goal, scope, dependencies, or verification, stop and return approval to pending before revising the plan.
7. Run every task verification check, then run Planning Revision Validation again. Write evidence only when both pass. Never change the approved Overview or Task Graph to record progress.

Canonicalize the project root and the nearest existing parent of `evidence/`; stop unless that parent equals or is contained by the project root. Create `evidence/` only after this check, then canonicalize it and require it to remain under the project root. Choose a safe `.md` basename without path separators, independently of the task ID, and create it exclusively inside `evidence/`. Include:

- frontmatter `task` encoded as a YAML string, `status: verified`, `verified_at` as a quoted ISO 8601 string, and `planning_revision` copied from the Approval Record
- `Result`, `Checks`, and `Artifacts` sections
- the exact checked artifact, command or test, or named human review and its result

Keep the record concise. A link or short result is evidence; copied logs and a second task description are not.

After writing verified evidence, read [Learning](learning.md) completely and apply it.
