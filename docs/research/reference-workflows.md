# Reference Workflow Findings

Created on 2026-08-09 and last reviewed on 2026-08-11. This note records precedent and failure signals. The current design decision is [ADR 0001](../../adr/0001-thin-first-version.md). The Addy Osmani review used commit [`7676817`](https://github.com/addyosmani/agent-skills/commit/7676817c12a1317454ae3898a0c5c1eacf5dd3d5), the Prime Agent review used commit [`71ca6cf`](https://github.com/PrimeIntellect-ai/prime-agent/commit/71ca6cfd1a2f7205ca0ec1baa65d10d0ed88f6e8), and pen.dev documentation was accessed on 2026-08-11.

## Finding

No reviewed project implements the complete autodev flow. [Matt Pocock Skills](https://github.com/mattpocock/skills) is the closest interaction precedent: `grilling` interviews along decision dependencies, `grill-with-docs` preserves selected context, and `to-tickets` asks the user to review task granularity and blocking relationships. It remains software-focused and does not supply an external personal Knowledge Root, a canonical cross-domain Project Overview, content-bound approval, or a learning-candidate lifecycle.

Autodev should reuse these interaction patterns, not adopt another framework or runtime.

## Useful precedents

| Source | Useful part | Boundary for autodev |
| --- | --- | --- |
| [Matt Pocock Skills](https://github.com/mattpocock/skills) | Dependency-aware interview, material ADRs, reviewed task breakdown | Reference the interaction contract; do not copy the full skill suite |
| [Addy Osmani Agent Skills](https://github.com/addyosmani/agent-skills) | Explicit assumptions, measurable success criteria, source-driven decisions, and structural, routing, and behavioral Skill evaluation | Keep one Autodev flow; do not import its full lifecycle, duplicate planning files, meta-router, personas, or arbitrary thresholds |
| [Prime Agent](https://github.com/PrimeIntellect-ai/prime-agent) | Immutable core plus supplemental knowledge, reviewable propose/apply refinement, bounded continuation, and release-artifact checks | Treat it as an Agent Host precedent; do not import its daemon, IPython runtime, scheduler, agent protocol, or automatic refinement |
| [pen.dev](https://docs.pen.dev/) | Editable `.pen` sources, headless CLI and local MCP access, and image or PDF export for visual verification | Treat Pen as an optional registered project tool, not an Autodev dependency or universal design choice |
| [Shape Up Pitch](https://basecamp.com/shapeup/1.5-chapter-06) | Reduce raw ideas to problem, appetite, solution, risks, and exclusions | Borrow the reduction questions, not the whole delivery method |
| [OpenSpec](https://github.com/Fission-AI/OpenSpec) and [Spec Kit](https://github.com/github/spec-kit) | Separate current truth from proposed change; clarify before planning | Avoid mandatory duplicate proposal, spec, design, and task artifacts |
| [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) | Markdown provenance, verification, lifecycle, permissive fields | Use a small compatible subset; do not require its wider ecosystem |
| [Karpathy LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) and [OpenWiki](https://github.com/langchain-ai/openwiki) | Separate raw sources from linked Markdown knowledge | Treat OpenWiki as a possible future producer, not a v1 dependency |
| [Semantica](https://github.com/semantica-agi/semantica) | Decision provenance, causal traces, and human-editable Markdown memory round trips | Consider as a future derived index or audit adapter; do not use it as a v1 dependency or approval gate |
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

### More routing can interfere with the Host

Addy Osmani Agent Skills users reported slow or conflicting behavior when a meta-router duplicated a capable Host's own routing in [#423](https://github.com/addyosmani/agent-skills/issues/423), and one user removed repetitive anti-rationalization instructions after they began interfering with newer models in [#433](https://github.com/addyosmani/agent-skills/issues/433).

Autodev should keep one thin entry point and load detailed references only when the current task needs them. A project configuration should record a selected tool's local purpose, not copy the tool or Skill's general capability description.

### Host discovery and project reentry

[Codex project instructions](https://developers.openai.com/codex/guides/agents-md) are discovered from `AGENTS.md` before work begins. [Codex Skills](https://developers.openai.com/codex/skills) can be selected implicitly from their descriptions and can be installed at repository or user scope. [Claude Code project memory](https://code.claude.com/docs/en/memory) loads `CLAUDE.md` at session start and documents importing an existing `AGENTS.md`; [Claude Code Skills](https://code.claude.com/docs/en/skills) also use descriptions for automatic loading.

Autodev should use those native discovery paths rather than require the user to remember a command. First setup adds one concise, managed routing block to `AGENTS.md` and an `@AGENTS.md` import for Claude Code. The marker identifies only Autodev planning and execution lifecycle requests, while the durable state remains in configured planning, approval, and evidence artifacts. Hooks and daemons remain unnecessary unless fresh-session tests show these native paths are insufficient.

### Source checks do not prove the distributed artifact

[Prime Agent issue #751](https://github.com/PrimeIntellect-ai/prime-agent/issues/751) reports a provider file missing from a stable distribution even though the source repository had broad CI. Addy Osmani Agent Skills similarly added link validation after installed Skill references broke in [#468](https://github.com/addyosmani/agent-skills/issues/468).

When a project ships a package, plugin, Skill, or other installable artifact, CI should create the real artifact, install it in a clean environment, and exercise its public entry point. Add a narrow regression check after a concrete packaging or reference failure rather than growing a speculative validator.

## Applied lessons

- Keep one canonical Overview and one approved Task Graph revision.
- Let the Agent Host choose execution tactics.
- Keep approval durable and content-bound rather than tied to a paused chat session.
- Separate approved planning inputs from mutable execution evidence.
- Keep the Knowledge Root user-owned and Markdown-readable.
- Keep general tool capabilities in their native Skill, MCP, CLI, or Host metadata. Record only the selected tool interface and project-specific purpose in project configuration.
- Verify a tool once when registering it. Do not persist credentials or mutable installation and authentication status, and do not silently substitute a different tool after failure.
- Add integrations, indexes, and notifications only in response to observed failures.

GitHub issues, release notes, and community discussions are qualitative evidence. They identify recurring failure modes but do not establish comparative effectiveness.
