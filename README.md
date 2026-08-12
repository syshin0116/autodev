# autodev

Knowledge-aware project design and execution for Agent Skills compatible hosts.

This repository contains the design record and the thin portable implementation. The design record remains authoritative.

- [Autodev Skill](SKILL.md)
- [Project Overview](docs/project-overview.md)
- [Current Task Graph](tasks.yaml)
- [Reference workflow findings](docs/research/reference-workflows.md)
- [Accepted design decision](adr/0001-thin-first-version.md)
- [Project template](templates/project)
- [Planning revision validation contract](docs/20-capability-contracts/planning-revision-validation.md)
- [Runtime mapping](docs/10-runtime-mapping.md)
- [Project contract verification](evidence/project-contract.md)
- [Planning Skill verification](evidence/knowledge-aware-planning-skill.md)
- [Execution and learning verification](evidence/approved-execution-and-learning.md)

`.autodev/` contains machine-readable configuration and approval state. When approved, `.autodev/approval.yaml` binds the exact Overview and task-source revision; changing approval-bound content invalidates it.

Validate an approved revision with `cargo run --locked --quiet --manifest-path <autodev-skill>/Cargo.toml -- <project-root>`.

Install this repository through an Agent Skills compatible Host, then invoke `autodev` with a rough idea. The Skill reaches a content-bound approval, then a later execution request runs one ready task, records verification evidence, and proposes only novel, sourced learning candidates to an authorized inbox.

Autodev itself is installed once per Host. On first use in a project, it adds only missing project-contract files and a small Host discovery marker. It checks selected Skills, CLIs, and MCP servers before registering their project-specific purpose, but does not install or authenticate them automatically. Missing setup runs only through the Host's existing mechanism after an explicit user request.
