---
task: "task-scoped-authorization"
status: verified
verified_at: "2026-08-16T03:33:41+09:00"
planning_revision:
  docs/project-overview.md: 872e6ddbf353ac5d6c820003041daac6257f77be88e76b10a592e96e21285d6a
  tasks.yaml: ef1fa0eaf797aa3356dba532b2820a6316f393b5e665ad68dab6f2d5950b2acb
operational_engine:
  provider: "OpenAI"
  name: "Codex"
  version: "host-managed, not exposed"
---

# Task-scoped authorization verification

## Result

Planning Revision Validation now separates rootless GitHub project approval from per-Issue authorization while retaining local and rooted GitHub compatibility. The Rust boundary validates project and raw task revisions, deterministic authorization generations, episode transitions, dependency readiness, merge evidence, and the required planning transition. The Skill keeps raw Issue content outside Agent input and stops GitHub execution without a trusted event.

Per-Issue serialization and durable state writes remain the responsibility of the next delivery adapter. This task returns and validates state decisions but does not claim to provide a lock, runner, or pull request automation.

## Checks

| Check | Result |
| --- | --- |
| `cargo run --locked --quiet --manifest-path Cargo.toml -- .` | `Planning revision valid.` |
| Approved Overview and Task Graph SHA-256 | Matched the recorded revision |
| `cargo test --locked --all-targets` | 20 passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| `cargo fmt --check` | Passed |
| Agent Skill format validation | Passed |
| `git diff --check` and forbidden em dash scan | Passed |

The regression cases cover project-policy projection, raw and filtered input digest separation, unrelated Issue isolation, actor and refusal gates, stable replay, generation changes, cancellation, suspension, supersession, dependency evidence and merge state, local compatibility, and the planning-transition stop marker.

## Artifacts

- [Autodev routing](../SKILL.md) and [execution contract](../references/execution.md)
- [Planning Revision Validation contract](../docs/20-capability-contracts/planning-revision-validation.md)
- [Rust capability](../src/lib.rs) and [CLI](../src/main.rs)
- [Regression coverage](../tests/planning_revision.rs)
