# Lint signal quality delta

## ADDED Requirements

### Requirement: Compiler-owned file diagnostics

r[octet.lint_signal_quality.source_ownership] File-level naming and module-count lints MUST inspect local admitted Rust source files without treating imported dependency files as consumer-owned files.

#### Scenario: Invalid local file remains actionable

- GIVEN a local admitted Rust source file violates the filename rule
- WHEN Octet checks its crate
- THEN the filename finding MUST remain actionable.

#### Scenario: Imported dependency files do not contaminate the consumer

- GIVEN rustc imports a dependency source file with an underscore filename
- AND the dependency shares the consumer workspace directory
- WHEN Octet checks the consumer crate
- THEN the dependency file MUST NOT produce a consumer filename finding or increase the consumer module count.

#### Scenario: Valid local file remains accepted

- GIVEN a local Rust source file satisfies the filename rule
- WHEN Octet checks its crate
- THEN Octet MUST NOT emit a filename finding for that file.
