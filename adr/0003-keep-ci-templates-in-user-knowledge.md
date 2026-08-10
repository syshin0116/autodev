---
status: accepted
date: 2026-08-10
runtime_example_superseded_by: 0005-bind-planning-validation-to-rust.md
---

# ADR 0003: Keep CI templates in user knowledge

## Context

Software projects need baseline CI soon after their stack and local verification commands are known. Recreating the workflow from memory wastes time, while copying an old workflow without checking it can preserve obsolete actions, commands, permissions, and runner assumptions.

GitHub offers organization workflow templates, reusable workflows, and composite actions. Organization templates require an organization-owned `.github` repository. Reusable workflows and remote composite actions create live cross-repository dependencies. Neither boundary is justified for a user-owned first version that must also remain portable across Agent Hosts and Git forges.

The [Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) already defines provenance, meaningful update time, verification events, lifecycle, and an absolute staleness date for Markdown knowledge. The [Agent Skills specification](https://agentskills.io/specification) establishes linked static assets as the appropriate place for reusable templates.

## Decision

Store each reusable CI template in a user-selected, read-only Knowledge Root as one Markdown knowledge record with linked workflow assets. The target project receives an adapted copy and owns it afterward.

Markdown records and linked assets remain canonical. A graph database or other search index is a rebuildable view whose entries preserve the source path and commit. It must not become the only copy of a template or decision.

The record uses these freshness fields:

- `created`: immutable ISO 8601 creation time, using the [DCMI Date Created](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/terms/created/) meaning
- `generated.at`: the current content's last meaningful change
- `verified`: source and asset verification events
- `stale_after`: the date on or after which the record requires refresh before reuse
- `sources`: official material supporting mutable claims

Before reuse, Autodev matches the record's applicability to the selected stack, forge, project commands, and constraints. It checks mutable claims against current official sources even when the record is not stale, cites adopted guidance in the Project Overview, and records rejected or adapted assumptions. It never writes back to the read-only Knowledge Root. A reusable correction becomes a learning candidate.

For GitHub Actions templates:

- grant only required `GITHUB_TOKEN` permissions
- pin remote actions to verified full commit SHAs
- use Dependabot for GitHub Actions references when ongoing update pull requests are wanted
- derive a baseline CI task after the stack and clean-checkout verification commands are known, before independent feature tasks become ready

The first stack-specific template is deferred to the approved reusable CI task. ADR 0005 retired the original Ruby example before publication.

## Considered options

### Generate every workflow from current documentation

This stays current but discards proven project knowledge and repeats avoidable decisions.

### Use a central reusable workflow

This propagates fixes centrally but couples every caller to another repository, its access policy, and its version lifecycle. Add it only after multiple projects need the same whole job.

### Use a GitHub organization workflow template

This is useful for organization-wide discovery and copy-on-create setup. It does not fit personal repositories without introducing an organization and remains GitHub-specific.

### Copy a sourced knowledge template

This is the selected option. It preserves user ownership, lets each project diverge, and keeps prior decisions challengeable.

## Consequences

- CI templates remain portable Markdown knowledge plus ordinary files.
- Creation, update, verification, and staleness have distinct dates.
- Projects do not inherit a hidden runtime dependency on Autodev or another repository.
- Official research remains necessary because dates and prior verification do not prove current validity.
- Template improvements wait for review as learning candidates instead of silently changing accepted knowledge.

## References

- [GitHub: Creating workflow templates](https://docs.github.com/en/actions/how-tos/reuse-automations/create-workflow-templates)
- [GitHub: Reusing workflows](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows)
- [GitHub: Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use)
- [GitHub: Keeping Actions up to date with Dependabot](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/auto-update-actions)
