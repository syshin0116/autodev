---
status: proposed
date: 2026-08-20
---

# ADR 0007: Run the delivery engine on a local host

## Context

ADR 0006 selected GitHub Agentic Workflows with Codex as the first delivery adapter, and that adapter now exists. Its agent job runs on a GitHub-hosted runner, which holds no operator session, so the engine can only authenticate with a metered provider API key. GitHub Agentic Workflows documents that provider OAuth, including a ChatGPT or Claude subscription, is not supported for either engine.

The operator already pays for engine subscriptions on a local machine and does not want a second metered bill for the same work. That cost is not a detail inside the approved engine policy; it decides whether the loop runs at all.

The trusted parts of the adapter do not have this problem. Authorization, the deterministic gates, and the durable episode record are cheap GitHub Actions work that needs no engine credential.

## Decision

Split the adapter by what each side is good at. GitHub keeps every trust decision and durable state. The engine runs on an operator-invoked local host.

- `autodev-authorize.yml` continues to run in GitHub Actions on an authorized ready event. It binds the actor and role at event time, decides the authorization through the Rust boundary, and persists the authorization record before any side effect.
- A committed local runner claims one authorized episode, gives the engine only the integrity-filtered projection, runs project-owned checks, and creates the branch and draft pull request itself. The engine receives no repository credential and performs no repository write.
- The selected engine is local Codex. The engine policy allows Anthropic and OpenAI so the engine can change without a new project revision, while the data-use boundary and cost class change to subscription execution on the operator's machine.
- The runner is started by hand. No scheduler, daemon, or background model session is added.

Delivery therefore advances only while the operator runs the command. GitHub holds the durable episode state between runs, so a machine that is asleep delays work rather than losing it.

The GitHub Agentic Workflow source, its compiled lock workflow, and the compiler drift check are removed rather than left disabled, because an inactive workflow that still reacts to events is a worse boundary than no workflow.

## Considered options

### Register a provider API key

This keeps the adapter exactly as built and remains the only option for delivery while the operator's machine is off. It was rejected for now because it bills separately from subscriptions the operator already holds, for the same work.

### Run the agentic workflow on a self-hosted runner

This keeps GitHub as the execution surface while using the local machine. It was rejected because the agentic workflow runs the engine inside a container whose `CODEX_HOME` is regenerated for MCP configuration, so subscription authentication is still not a supported path, and a self-hosted runner attached to a public repository can execute fork pull request code on the operator's machine.

### Move everything local

Authorizing in the same place the work runs would remove the trusted event boundary that binds the authorizing actor and role. Rejected.

## Consequences

- ADR 0006's expectation that delivery continues without another local session is narrowed to authorization and gates. Implementation waits for the operator.
- Engine cost moves from metered API billing to an existing subscription, and no engine credential is stored in the repository.
- The gh-aw dependency, its preview surface, and the compiled lock workflow leave the repository.
- A later revision can restore hosted execution by adding the provider key back, because the authorization record and the episode contract do not change.
- Cancellation, supersession, and question handling still need the transition path; a local runner does not remove that work.
