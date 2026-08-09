---
task: knowledge-aware-planning-skill
status: verified
verified_at: 2026-08-10
---

# Knowledge-aware planning Skill verification

A fresh Agent Host used the root [Autodev Skill](../SKILL.md) to rebuild autodev in an empty temporary project. It received only the [rough request](../test/fixtures/planning-skill/request.md), the selected [prior record](../test/fixtures/planning-skill/knowledge/previous-autodev-retrospective.md), and the target paths.

The Agent asked three grouped questions about the first-release boundary, local knowledge retrieval, and compatibility with the archived implementation. The answers selected an approved handoff instead of a runtime, on-demand read-only Markdown roots instead of an index, software and non-software support, and a clean break from the archive.

The Agent then produced the captured [Project Overview](../test/fixtures/planning-skill/project/docs/project-overview.md) and [Task Graph](../test/fixtures/planning-skill/project/tasks.yaml), incorporated one requested citation correction, showed the complete revision again, asked a separate approval question, recorded the explicit answer in the [Approval Record](../test/fixtures/planning-skill/project/.autodev/approval.yaml), validated the revision, and stopped before execution.

## Results

| Check | Result |
| --- | --- |
| Relevant prior knowledge affects the plan | Speculative orchestration and detailed standing rules are deferred |
| Prior context is nonbinding | Legacy formats, architecture, and unadopted decisions are explicitly excluded |
| Progressive source trace | The Overview links directly to the selected Markdown record |
| Interview closure | `Open questions` is `None.` after material answers were supplied |
| Task derivation | Every task links to a local Overview section and carries verification checks |
| Exact-byte approval | Both planning-file SHA-256 values were identical before and after approval and match the Approval Record |
| Read-only knowledge | The selected record kept SHA-256 `4503e9f7...` before and after planning |
| Handoff boundary | No execution evidence or project output was created |

## Checks

- `ruby scripts/validate_project.rb test/fixtures/planning-skill/project`
- `ruby -Itest test/planning_skill_test.rb`
- Agent Skills format validation with `quick_validate.py`
