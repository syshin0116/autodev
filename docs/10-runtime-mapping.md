# Runtime Mapping

Concrete bindings live here so the design contract remains stable when a binding changes.

| Capability | Current binding | Entry point | Evidence |
| --- | --- | --- | --- |
| [Planning Revision Validation](20-capability-contracts/planning-revision-validation.md) | Rust 1.85+ with locked crates; `gh api` for GitHub; fresh Host-native MCP reads for Kaneo | `cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml --`; provider modes are in the capability contract | `tests/planning_revision.rs`, `tests/kaneo_projection.rs`, `tests/cli.rs` |
| [Delivery Adapter](30-delivery-adapter.md) | GitHub Actions for authorization; local Codex CLI for implementation | `.github/workflows/autodev-authorize.yml` decides, `scripts/autodev-deliver.sh` implements | Workflow runs, the authorization record comment on each issue, and `tests/agent_input.rs` |
