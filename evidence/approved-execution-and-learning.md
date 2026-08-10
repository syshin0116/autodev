---
task: approved-execution-and-learning
status: verified
verified_at: "2026-08-10"
planning_revision:
  docs/project-overview.md: d6581f19053aa0042560006373a80165df5b115540477aae52c635c9ed2d1994
  tasks.yaml: d7cc1075a13bb7961aa901601f166da1f15fb3154c2ecc8d29260f642fd14620
---

# Approved execution and learning verification

Two fresh Agent runs on the same Agent Host received the same approved one-task project. One project retained the approved bytes. The other added one line to the Overview after approval.

The unchanged project passed validation before and after execution, created the [check-in sheet](../test/fixtures/execution-learning/project/output/check-in.md), and recorded [task evidence](../test/fixtures/execution-learning/project/evidence/build-check-in-sheet.md) bound to the current planning hashes. Its candidate inbox retained the existing dismissed identifier lesson and added one [pending CSV candidate](../test/fixtures/execution-learning/candidate-inbox/parse-quoted-csv-fields.md). No duplicate of the dismissed lesson was created.

The altered project stopped at Project Validation with `approved planning file changed: docs/project-overview.md`. It created no output, evidence, or candidate.

Two additional fresh runs covered the review regressions. A requested dependency-blocked task stopped instead of falling back to another ready task. A task with evidence from an older approved revision reported the conflict instead of rerunning. Neither run changed a file.

Minitest validates the captured artifact contract and Project Validation. The fresh Agent runs, not Minitest, provide the observed execution behavior.

## Checks

| Check | Result |
| --- | --- |
| Root and both captured project validations | Passed |
| Full Minitest suite | 6 runs, 51 assertions, no failures or errors |
| Changed planning bytes | The validator regression rejects them; the fresh altered run stopped before writes |
| Captured source trace | Changing output ID `001` to `1` made the focused test fail |
| Agent Skills format validation | Passed |
| Approved planning bytes | Unchanged by successful execution |

## Artifacts

- [Autodev Skill](../SKILL.md)
- [Execution fixture](../test/fixtures/execution-learning)
- [Regression test](../test/planning_skill_test.rb)
