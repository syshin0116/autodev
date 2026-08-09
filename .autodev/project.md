---
id: autodev
status: approved
approval: user-approved-in-chat-2026-08-09
---

# Goal

Turn an opportunity or free-form idea into a decision-complete project source through knowledge-aware interviewing, reduce it to the minimum useful scope, derive a verifiable task graph, and execute it after explicit user approval.

# Users

People using Codex or Claude Code who want to build their own private, reusable body of decisions, preferences, procedures, and lessons.

# Inputs

- Free-form conversation about something the user wants to do
- External opportunities such as competitions, grants, and requests for proposals
- Source files and links supplied by the user or a future discovery tool
- User-owned knowledge stored outside autodev

# Deliverables

- A concise project source with resolved decisions and no material open questions
- A task graph whose tasks have dependencies, completion criteria, and verification
- The requested artifact, including software, proposals, presentations, research, or content
- Reviewable knowledge candidates produced from decisions and outcomes

# Core flow

```text
remember -> interview -> reduce -> plan -> approve -> execute -> verify -> learn
```

# Decisions

- Ship autodev as a reusable Agent Skill for Codex and Claude Code, not as a new chat UI or model runtime.
- Keep autodev, user knowledge, project state, and run history separate.
- Use connected knowledge to improve questions and assess decisions, not to force past answers onto new contexts.
- Keep the project source canonical. Generate task graphs and artifact-specific documents from it.
- Require explicit user approval of the project source and task graph before execution.
- Bind approval to the approved content so later changes require another approval.
- Return new experience as knowledge candidates. Never promote it to accepted knowledge without review.
- Preserve the current repository as a renamed, archived historical record. Reuse the `autodev` name for the new repository.
- Use the autodev skill itself as the first end-to-end artifact, so the project dogfoods its own interview, approval, task, verification, and learning flow.
- Require a knowledge candidate to state its context, application conditions, and source or run evidence before a user can accept it.
- Let each user register multiple Markdown knowledge roots and select the readable roots per project.
- Require each project to select exactly one writable candidate inbox. Other knowledge roots remain read-only.
- Require one project overview as the stable entry point for every project.
- Make each material decision traceable from project context through compared options and rationale to observed outcome and evidence.
- Apply progressive disclosure: overview first, decision detail second, raw source and run evidence last.
- Treat decisions explicitly included in an approved project source as accepted knowledge without a second approval.
- Require immediate accept, merge, discard, or defer review for newly inferred learnings at project close.
- Surface deferred candidates from related projects during the next relevant remember step instead of relying on a passive inbox.

# Constraints

- User knowledge must remain user-owned and outside the distributed autodev repository.
- Every project knowledge set must have exactly one overview that remains understandable without opening every linked record.
- Another user must be able to install autodev and connect their own knowledge.
- The first version must work with Markdown knowledge directories without a graph database or vector store.
- The first version must not require a standalone server, custom chat UI, or orchestrator.
- The skill must keep its main instructions concise and load detailed guidance only when needed.
- The system must not start execution while material questions remain unresolved.

# Success criteria

- A user can start with either a free-form idea or supplied opportunity material.
- The interview reuses relevant prior knowledge without silently treating it as binding.
- The resulting project source contains only information that affects the deliverable or its verification.
- Every task traces to the approved project source and has a runnable or human-verifiable completion check.
- Execution cannot start without explicit approval of the current project source and task graph.
- A completed run produces evidence and reviewable knowledge candidates.
- The same core skill works in both Codex and Claude Code on one real project.
- A reader can start at a project overview and trace why a decision was made, what was compared, and what happened afterward.
- Approved project decisions do not accumulate in a second approval queue, and deferred learnings resurface when relevant.

# Non-goals for the first version

- Automatic opportunity discovery
- A hosted knowledge service
- A graph database or embedding pipeline
- Multi-agent organizations or independent reviewer identities
- Automatic merge infrastructure
- Self-improvement and trend-monitoring systems
- Dashboards, reporting layers, or tool-swap capability matrices

# References

- The current `autodev` repository is historical source material, not the implementation base.
- The conversation that produced this document is the first raw interview record.

# Open questions

None.
