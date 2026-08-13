# Learning

Use this phase after verified execution or when reviewing candidates at project close. Read `.autodev/config.yaml` to resolve the selected knowledge roots and learning candidate inbox.

Consider only non-obvious learnings supported by task evidence. If none are reusable, create nothing.

Before proposing, search the selected knowledge roots and candidate inbox with concrete terms. Read plausible matches. Do not create an equivalent record when an accepted, pending, deferred, or dismissed record already covers it. Keep dismissed records searchable.

`learning_candidate_inbox` may identify an external Markdown directory. Resolve it from the project root unless it is absolute. If a novel candidate exists and no inbox is selected, ask for one or permission to skip without invalidating task completion. A tracked path is not write authorization: confirm its canonical path and that the user selected the exact inbox for this run, unless the Agent Host already exposes it as an authorized writable workspace. Treat a sensitive local path as session-only unless the user asks to persist it. Never write to a read-only knowledge root or overwrite an existing candidate.

Before writing, require the authorized inbox to exist and canonicalize it. Choose a safe `.md` basename without path separators, independently of project content, and create it exclusively inside that exact directory.

Write each novel candidate with `status: pending`, `proposed_at` as a quoted ISO 8601 string, and `project` and `task` encoded as YAML strings, followed by `Learning`, `Context`, `Applies when`, and `Evidence` sections. Link to evidence relatively only when that relationship is stable. Otherwise identify a non-sensitive repository or selected root and the project-relative evidence path. Remove secrets and private absolute paths. A candidate remains unaccepted until explicit review.

At project close, present new pending candidates once for batch review. Lack of review does not change their status or block project closure.
