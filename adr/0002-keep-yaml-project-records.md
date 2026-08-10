---
status: accepted
date: 2026-08-09
approval: user-selected-option-3-in-chat
---

# ADR 0002: Keep project machine records in YAML

## Context

The first planning revision validator must parse the existing Task Graph, configuration, and Approval Record without adding a package dependency. The chosen validation runtime must not leak into the durable project contract because it can be replaced independently.

## Decision

Keep the machine records in YAML and bind Planning Revision Validation to a Runtime whose standard library provides safe YAML parsing and SHA-256 hashing.

The abstract behavior is defined by the [Planning Revision Validation Capability](../docs/20-capability-contracts/planning-revision-validation.md). The current concrete binding is recorded only in the [Runtime Mapping](../docs/10-runtime-mapping.md).

## Considered options

### Change the machine record format

This would avoid a parser dependency in another Runtime but would invalidate the approved planning artifact and discard the existing readable format.

### Add a YAML package dependency

This preserves the format but adds installation and version management before the first useful validation.

### Use YAML support from the selected Runtime

This is the selected option. It preserves the approved records and adds no package dependency.

## Consequences

- Existing YAML files and approval hashes remain valid.
- The planning revision validator requires the capability named in the Runtime Mapping.
- A Host without that capability needs a compatible planning revision validator binding, not a change to project artifacts.
- YAML is parsed safely with aliases disabled.
