<div align="center">

# autodev

**Turn a rough idea and selected knowledge into an approved plan your Agent Host can execute.**

[![Agent Skill](https://img.shields.io/badge/Agent-Skill-6f42c1)](SKILL.md)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-dea584?logo=rust)](Cargo.toml)

[How it works](#how-it-works) · [Quick start](#quick-start) · [Project contract](#project-contract) · [Read the design](#read-the-design)

</div>

Autodev is an Agent Skill for knowledge-aware project planning and execution. It interviews only while an unresolved answer can change the plan, writes a concise Project Overview and dependency-aware Task Graph, and binds approval to that exact revision. On a later request, the Agent Host executes one ready task and records verification evidence.

The Rust binary validates planning revisions. It is not a separate planner, task runner, or Agent runtime.

## How it works

```text
rough idea + selected Markdown knowledge
                    |
                    v
            focused interview
                    |
                    v
       Project Overview + Task Graph
                    |
                    v
       explicit revision-bound approval
                    |
              later request
                    v
          execute one ready task
                    |
                    v
       verify and record evidence
                    |
                    v
      propose reusable learning for review
```

### Three guarantees

1. **Approval is content-bound.** A changed Overview, local Task Graph, or approval-bound GitHub Issue projection requires a new approval.
2. **Execution does not rewrite the plan.** Results and checks live in separate evidence records.
3. **Knowledge stays under user control.** Selected Markdown roots are read-only, and reusable learnings remain candidates until reviewed.

## Quick start

### Requirements

- Git
- Rust 1.85+ and Cargo
- An Agent Host with local Skill support
- Optional: authenticated GitHub CLI (`gh`) for a GitHub Issues task source

### Install in Codex

```sh
mkdir -p "$HOME/.agents/skills"
git clone https://github.com/syshin0116/autodev.git "$HOME/.agents/skills/autodev"
```

Open the target project in Codex and start with a rough idea:

```text
$autodev Plan a volunteer onboarding workshop. Use a local task file.
```

Autodev adds only missing project-contract files, resolves material planning questions, and presents the complete planning revision. After reviewing it, approve in a separate response:

```text
I approve this exact planning revision for execution.
```

Autodev records and validates the approval, then stops. Execute work in a later request:

```text
$autodev Execute the next ready task.
```

Autodev revalidates the revision, executes and verifies one ready task through the Agent Host, and records evidence. It does not install project tools or own their credentials.

## Project contract

| Artifact | Responsibility |
|---|---|
| [`SKILL.md`](SKILL.md) | Planning, approval, execution, and learning workflow |
| `docs/project-overview.md` | Canonical project intent, decisions, scope, and success criteria |
| `tasks.yaml` or GitHub Issues | Approval-bound outcomes, dependencies, references, and checks |
| `.autodev/config.yaml` | Project paths and selected task source |
| `.autodev/approval.yaml` | Approver metadata and exact planning revision digests |
| `evidence/` | Verified task results without mutating the approved plan |

### Validate an approved revision

```sh
cargo run --locked --quiet \
  --manifest-path "$HOME/.agents/skills/autodev/Cargo.toml" \
  -- /absolute/path/to/project
```

A valid revision prints `Planning revision valid.` GitHub projection commands and failure conditions are documented in [Planning Revision Validation](docs/20-capability-contracts/planning-revision-validation.md).

## Read the design

| Document | Contents |
|---|---|
| [Project Overview](docs/project-overview.md) | Goal, boundaries, current decisions, and proposed work |
| [Autodev Skill](SKILL.md) | Complete Host-facing behavior |
| [Planning Revision Validation](docs/20-capability-contracts/planning-revision-validation.md) | Local and GitHub task-source validation contract |
| [Runtime Mapping](docs/10-runtime-mapping.md) | Capability to Rust and GitHub CLI mapping |
| [Reference Workflows](docs/research/reference-workflows.md) | Reviewed precedents, failure reports, and adopted lessons |
| [Architecture Decisions](adr/) | Decisions active in this repository |
| [Verification Evidence](evidence/) | Captured checks for implemented capabilities |

## Development

```sh
cargo test --locked --all-targets
```

Autodev deliberately reuses the Agent Host instead of adding a custom model runtime, server, chat UI, or knowledge database. The [archived predecessor](https://github.com/syshin0116/autodev-archive) remains historical context, not the active implementation.
