---
id: autodev
status: approved
approval: user-approved-in-chat-2026-08-09
---

# Goal

Turn an opportunity or free-form idea into a decision-complete Project Overview through knowledge-aware interviewing, reduce it to the minimum useful scope, derive a verifiable Task Graph, and execute it after explicit user approval.

# Users

People using an Agent Skills compatible Agent Host who want to build their own private, reusable body of decisions, preferences, procedures, and lessons.

# Inputs

- Free-form conversation about something the user wants to do
- External opportunities such as competitions, grants, and requests for proposals
- Source files and links supplied by the user or a future discovery tool
- User-owned knowledge stored outside autodev

# Deliverables

- A concise Project Overview with resolved decisions and no material open questions
- A Task Graph whose tasks have dependencies, completion criteria, and verification
- The requested artifact, including software, proposals, presentations, research, or content
- Reviewable knowledge candidates produced from decisions and outcomes

# Core flow

```text
remember -> interview -> reduce -> plan -> approve -> execute -> verify -> learn
```

# Decisions

- Ship autodev as a portable Agent Skill, not as a new chat UI, model runtime, or product-specific integration.
- Treat Codex and Claude Code only as the first Agent Host compatibility examples.
- Keep autodev, user knowledge, project state, and run history separate.
- Use connected knowledge to improve questions and assess decisions, not to force past answers onto new contexts.
- Keep the Project Overview canonical. Generate the Task Graph and artifact-specific documents from it.
- Require explicit user approval of the Project Overview and Task Graph before execution.
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
- Treat decisions explicitly included in an approved Project Overview as accepted knowledge without a second approval.
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
- Every generated artifact must omit content that only repeats its title, surrounding structure, or already established context.
- Every generated artifact must disclose the minimum decision-relevant summary first and link to detail and Evidence only where needed.

# Success criteria

- A user can start with either a free-form idea or supplied opportunity material.
- The interview reuses relevant prior knowledge without silently treating it as binding.
- The resulting Project Overview contains only information that affects the deliverable or its verification.
- Every task traces to the approved Project Overview and has a runnable or human-verifiable completion check.
- Execution cannot start without explicit approval of the current Project Overview and Task Graph.
- A completed run produces evidence and reviewable knowledge candidates.
- The same core skill works in at least two Agent Hosts on one real project.
- A reader can start at a project overview and trace why a decision was made, what was compared, and what happened afterward.
- Approved project decisions do not accumulate in a second approval queue, and deferred learnings resurface when relevant.
- Removing an obvious or duplicated statement does not reduce the artifact's ability to guide action or verify completion.
- A reader can move from summary to Decision Record to Evidence without loading unrelated detail.

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

# Canonical terms

- `Autodev`: the complete Agent Skill and workflow
- `Agent Skill`: the portable SKILL.md package
- `Agent Host`: an external product that loads and executes the Agent Skill
- `Host Capability`: a capability an Agent Host must provide for autodev
- `Host Compatibility Check`: evidence that autodev works in a specific Agent Host
- `Project Overview`: the mandatory top-level project entry point
- `Decision Record`: context, compared options, rationale, outcome, and evidence for a material choice
- `Task Graph`: approved tasks and their dependencies
- `Run Record`: raw execution and verification evidence
- `Knowledge Root`: a user-owned readable knowledge directory
- `Candidate Inbox`: the single writable destination for Knowledge Candidates
- `Knowledge Candidate`: an inferred learning that is not yet accepted
- `Approval Record`: user approval bound to the current Project Overview and Task Graph
- `Evidence`: a source, file, link, or Run Record supporting a decision or outcome

# Open questions

None.
