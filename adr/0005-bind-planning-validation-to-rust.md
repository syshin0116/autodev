---
status: accepted
date: 2026-08-10
approval: user-selected-rust-in-chat
---

# ADR 0005: Bind planning validation to Rust

## Context

The Ruby validator proved the local planning contract quickly. GitHub Issue Graph support adds recursive graph reads, deterministic JSON projection, and a larger validation surface. The user selected Rust for the durable binding.

ADR 0002 assumed a Runtime whose standard library supplied both YAML parsing and SHA-256. Rust supplies neither, so the runtime change cannot preserve that implementation constraint without changing the approved YAML records.

## Decision

Implement Planning Revision Validation as one Rust package with a committed `Cargo.lock`. Keep YAML records unchanged and use narrowly scoped crates for typed YAML, JSON, and SHA-256. Use the installed `gh` CLI only for authenticated GitHub API transport.

The YAML parser bounds recursion and alias repetition. Typed deserialization rejects values that do not match the planning record shapes. This replaces ADR 0002's no-package and aliases-disabled requirements while preserving its YAML-format decision.

The Ruby-specific CI example in ADR 0003 is retired. The approved reusable CI task will choose and verify its own stack-specific template.

## Consequences

- Rust 1.85 or newer and Cargo are required to build the validator.
- Locked dependencies make the binding reproducible without making Rust part of the abstract capability contract.
- The repository contains no second validator implementation.
- A configured GitHub source also requires an authenticated `gh` CLI with readable Issues, sub-issues, and dependency endpoints.
- For `N` tasks, the first REST reader makes at least `3N + 2` requests per validation before pagination. Replace it with a measured bulk-read design only when a real graph reaches that limit.

## References

- [Cargo package layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [yaml_serde](https://github.com/yaml/yaml-serde)
- [GitHub CLI API manual](https://cli.github.com/manual/gh_api)
