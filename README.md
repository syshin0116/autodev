# autodev

Knowledge-aware project design and execution for Agent Skills compatible hosts.

This repository contains the design record and the thin portable implementation. The design record remains authoritative.

- [Project Overview](docs/project-overview.md)
- [Current Task Graph](tasks.yaml)
- [Reference workflow findings](docs/research/reference-workflows.md)
- [Accepted design decision](adr/0001-thin-first-version.md)
- [Project template](templates/project)
- [Project validation contract](docs/20-capability-contracts/project-validation.md)
- [Runtime mapping](docs/10-runtime-mapping.md)
- [Project contract verification](evidence/project-contract.md)

`.autodev/` contains machine-readable configuration and approval state. The current planning revision is approved in `.autodev/approval.yaml`; changing either planning artifact invalidates that approval.
