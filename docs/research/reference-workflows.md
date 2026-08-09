# Reference Workflow Findings

Reviewed on 2026-08-09. This note records precedent and failure signals. The current design decision is [ADR 0001](../../adr/0001-thin-first-version.md).

## Finding

No reviewed project implements the complete autodev flow. [Matt Pocock Skills](https://github.com/mattpocock/skills) is the closest interaction precedent: `grilling` interviews along decision dependencies, `grill-with-docs` preserves selected context, and `to-tickets` asks the user to review task granularity and blocking relationships. It remains software-focused and does not supply an external personal Knowledge Root, a canonical cross-domain Project Overview, content-bound approval, or a learning-candidate lifecycle.

Autodev should reuse these interaction patterns, not adopt another framework or runtime.

## Useful precedents

| Source | Useful part | Boundary for autodev |
| --- | --- | --- |
| [Matt Pocock Skills](https://github.com/mattpocock/skills) | Dependency-aware interview, material ADRs, reviewed task breakdown | Reference the interaction contract; do not copy the full skill suite |
| [Shape Up Pitch](https://basecamp.com/shapeup/1.5-chapter-06) | Reduce raw ideas to problem, appetite, solution, risks, and exclusions | Borrow the reduction questions, not the whole delivery method |
| [OpenSpec](https://github.com/Fission-AI/OpenSpec) and [Spec Kit](https://github.com/github/spec-kit) | Separate current truth from proposed change; clarify before planning | Avoid mandatory duplicate proposal, spec, design, and task artifacts |
| [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) | Markdown provenance, verification, lifecycle, permissive fields | Use a small compatible subset; do not require its wider ecosystem |
| [Karpathy LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) and [OpenWiki](https://github.com/langchain-ai/openwiki) | Separate raw sources from linked Markdown knowledge | Treat OpenWiki as a possible future producer, not a v1 dependency |
| [Link](https://github.com/gowtham0992/link) | Review-gated memory, provenance, deduplication, dismissal, supersession | Borrow candidate semantics without building a memory product |
| [Beads](https://github.com/gastownhall/beads) | Dependency-aware tasks and a ready frontier | Borrow graph semantics; keep the first Task source as a file |
| [Agent Skills](https://github.com/agentskills/agentskills) | Portable package and progressive loading | Keep the core self-contained and avoid Host-specific chaining |

## Repeated failure signals

### Interview state and decisions disappear

Users reported excessive questioning, loss of the pending decision frontier after context compaction, and resolved decisions failing to reach implementation artifacts. See Matt Pocock Skills issues [#274](https://github.com/mattpocock/skills/issues/274), [#338](https://github.com/mattpocock/skills/issues/338), and [#341](https://github.com/mattpocock/skills/issues/341).

Autodev should ask only while an answer can alter scope, dependencies, or verification. It may checkpoint the unresolved frontier as machine state, but the frontier is not another durable project document.

### More documents create more drift

OpenSpec users requested a supported way to repair specifications during implementation in issues [#684](https://github.com/Fission-AI/OpenSpec/issues/684) and [#821](https://github.com/Fission-AI/OpenSpec/issues/821). Spec Kit users also identified a missing post-implementation diagnosis loop in [#442](https://github.com/github/spec-kit/issues/442).

Autodev should keep one canonical Overview, link only material Decision Records, and store evidence without copying the plan.

### Generated knowledge decays without curation

OpenWiki reports include cross-page drift and broken anchors in [#372](https://github.com/langchain-ai/openwiki/issues/372), plus a generated 19-page wiki with 152 unresolved links among 169 links in [#602](https://github.com/langchain-ai/openwiki/issues/602).

Autodev should keep raw evidence reachable, lint structural links, and never promote generated knowledge automatically.

### A review inbox accumulates noise

[Link v2.1.0](https://github.com/gowtham0992/link/releases/tag/v2.1.0) records duplicate and false memory proposals found during maintainer dogfooding. The response included cross-session deduplication, a dismissal ledger, batch review, and revision rather than overwrite.

Autodev should review candidates in a batch, remember dismissals, and resurface deferred candidates only when relevant.

### Task infrastructure can exceed the task

[Beads](https://github.com/gastownhall/beads) demonstrates the value of persistent dependency edges, while [community criticism](https://news.ycombinator.com/item?id=46669791) shows the maintenance cost when task state grows into its own platform.

Autodev needs dependency and readiness semantics, not a Task database in the first version.

## Applied lessons

- Keep one canonical Overview and one approved Task Graph revision.
- Let the Agent Host choose execution tactics.
- Keep approval durable and content-bound rather than tied to a paused chat session.
- Separate approved planning inputs from mutable execution evidence.
- Keep the Knowledge Root user-owned and Markdown-readable.
- Add integrations, indexes, and notifications only in response to observed failures.

GitHub issues, release notes, and community discussions are qualitative evidence. They identify recurring failure modes but do not establish comparative effectiveness.
