# autodev

Knowledge-aware project design and execution for Agent Skills compatible hosts.

This repository contains the design record and the thin portable implementation. The design record remains authoritative.

- [Autodev Skill](SKILL.md)
- [Project Overview](docs/project-overview.md)
- [Current Task Graph](tasks.yaml)
- [Reference workflow findings](docs/research/reference-workflows.md)
- [Accepted design decision](adr/0001-thin-first-version.md)
- [Project template](templates/project)
- [Project validation contract](docs/20-capability-contracts/project-validation.md)
- [Runtime mapping](docs/10-runtime-mapping.md)
- [Project contract verification](evidence/project-contract.md)
- [Planning Skill verification](evidence/knowledge-aware-planning-skill.md)
- [Execution and learning verification](evidence/approved-execution-and-learning.md)

`.autodev/` contains machine-readable configuration and approval state. The current planning revision is approved in `.autodev/approval.yaml`; changing either planning artifact invalidates that approval.

Install this repository through an Agent Skills compatible Host, then invoke `autodev` with a rough idea. The Skill reaches a content-bound approval, then a later execution request runs one ready task, records verification evidence, and proposes only novel, sourced learning candidates to an authorized inbox.
