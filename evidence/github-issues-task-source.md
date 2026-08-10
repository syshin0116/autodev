---
task: "github-issues-task-source"
status: verified
verified_at: "2026-08-10T20:55:10+09:00"
planning_revision:
  docs/project-overview.md: edb524e8119b5d5a797a8b85ddd3817238c196cd32d335352e77e4c9c40cca80
  tasks.yaml: 01c5714fe91b386bc3274905372df860ebfa7c5d87fd82d4aad24fbd22fd2492
---

## Result

Added a Rust planning revision validator that supports either the approved local Task Graph or one GitHub Issue Graph without changing this repository's active task source.

## Checks

- Rust 1.85 compiled the locked package.
- `cargo test --locked --all-targets` passed 13 tests, including the fixed projection digest, validated execution snapshot, recursive order, native dependencies, metadata exclusion, fail-closed API cases, local compatibility, and captured Skill artifacts.
- Clippy passed with warnings denied, formatting was clean, and the current approved local revision validated.
- Four isolated make-it-fail mutations were rejected: missing task bodies, missing local outcomes, missing pagination slurp, and skipped approval status validation.

## Artifacts

- [Planning revision validator](../src/lib.rs)
- [Regression tests](../tests/planning_revision.rs)
- [GitHub task template](../.github/ISSUE_TEMPLATE/autodev-task.md)
- [Rust binding decision](../adr/0005-bind-planning-validation-to-rust.md)
