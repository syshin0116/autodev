# Runtime Mapping

Concrete bindings live here so the design contract remains stable when a binding changes.

| Capability | Current binding | Entry point | Evidence |
| --- | --- | --- | --- |
| [Planning Revision Validation](20-capability-contracts/planning-revision-validation.md) | Ruby standard library YAML and SHA-256 support, checked with Ruby 2.6.10 | `scripts/validate_planning_revision.rb` | `test/validate_planning_revision_test.rb` |
