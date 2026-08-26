---
task: knowledge-aware-planning-skill
status: verified
verified_at: "2026-08-10"
planning_revision:
  docs/project-overview.md: a4dad5f38d7701d5cf82953296c087ee6112f5bd7bb5edd59eb04b6f344b1259
  tasks.yaml: 3b3bc02f484c9494ebc5b9913f8264c7944d48e94509b0ffe5e38a86b5adb4c6
---

# Knowledge-aware planning Skill verification

A fresh Agent Host used the root [Autodev Skill](../SKILL.md) to rebuild autodev in an empty temporary project. It received only the [rough request](../test/fixtures/planning-skill/request.md), the selected [prior record](../test/fixtures/planning-skill/knowledge/previous-autodev-retrospective.md), and the target paths.

The Agent asked three grouped questions about the first-release boundary, local knowledge retrieval, and compatibility with the archived implementation. The answers selected a reviewed handoff instead of a runtime, on-demand read-only Markdown roots instead of an index, software and non-software support, and a clean break from the archive.

The Agent then produced the captured [Project Overview](../test/fixtures/planning-skill/project/docs/project-overview.md) and [Task Graph](../test/fixtures/planning-skill/project/tasks.yaml), incorporated one requested citation correction, showed the complete revision again, asked a separate recording question, and stopped before execution. The current contract records accepted local planning files through Git instead of a duplicate approval file.

## Results

| Check | Result |
| --- | --- |
| Relevant prior knowledge affects the plan | Speculative orchestration and detailed standing rules are deferred |
| Prior context is nonbinding | Legacy formats, architecture, and unadopted decisions are explicitly excluded |
| Progressive source trace | The Overview links directly to the selected Markdown record |
| Interview closure | `Open questions` is `None.` after material answers were supplied |
| Task derivation | Every task links to a local Overview section and carries verification checks |
| Committed planning state | Validation accepts the tracked fixture and rejects an uncommitted planning change |
| Read-only knowledge | The selected record was unchanged before and after planning |
| Handoff boundary | No execution evidence or project output was created |

## Checks

- `cargo test --locked --test planning_revision captured_skill_artifacts_keep_the_planning_and_learning_contract`
- Agent Skills format validation with `quick_validate.py`
