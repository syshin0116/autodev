# Runtime Mapping

Concrete bindings live here so the design contract remains stable when a binding changes.

| Capability | Current binding | Entry point | Evidence |
| --- | --- | --- | --- |
| [Planning Revision Validation](20-capability-contracts/planning-revision-validation.md) | Rust 1.85+ with locked crates; `gh api` for a configured GitHub source | `cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml --`; project and rooted graph modes are in the capability contract | `tests/planning_revision.rs` |
