# Runtime Mapping

Concrete bindings live here so the design contract remains stable when a binding changes.

| Capability | Current binding | Entry point | Evidence |
| --- | --- | --- | --- |
| [Project validation](20-capability-contracts/project-validation.md) | Ruby standard library YAML and SHA-256 support, checked with Ruby 2.6.10 | `scripts/validate_project.rb` | `test/validate_project_test.rb` |
