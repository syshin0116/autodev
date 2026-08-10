---
task: project-contract
status: verified
verified_at: "2026-08-09"
planning_revision:
  docs/project-overview.md: d6581f19053aa0042560006373a80165df5b115540477aae52c635c9ed2d1994
  tasks.yaml: d7cc1075a13bb7961aa901601f166da1f15fb3154c2ecc8d29260f642fd14620
---

# Project Contract Verification

The current approved project passes the [Project Validation Capability](../docs/20-capability-contracts/project-validation.md) using the binding in the [Runtime Mapping](../docs/10-runtime-mapping.md).

## Checks

| Check | Result |
| --- | --- |
| `ruby scripts/validate_project.rb` | Passed |
| `ruby -Itest test/validate_project_test.rb` | 4 runs, 13 assertions, no failures or errors |
| Ruby syntax checks | Passed |
| Safe parsing of every YAML file | Passed |
| Local Markdown links | All resolved |
| Whitespace and em dash check | Passed |

The tests cover an approved fixture, unresolved questions, duplicate and unknown dependencies, self-dependency and multi-task cycles, missing verification, missing or pending approval, both planning-file hash mismatches, and a configured path escaping the project root.

## Artifacts

- [Validator](../scripts/validate_project.rb)
- [Tests](../test/validate_project_test.rb)
- [Project template](../templates/project)
- [ADR 0002](../adr/0002-keep-yaml-project-records.md)
