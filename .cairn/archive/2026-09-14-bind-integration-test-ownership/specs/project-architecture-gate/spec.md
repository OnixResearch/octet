## ADDED Requirements

### Requirement: Integration-test roles are compiler-bound
r[octet.project_architecture_gate.test_ownership]

A selected independent Cargo integration-test unit MAY use an explicit matching test source role.
Its source root, target kind, configuration, and membership MUST be bound to compiler selection and checked during artifact replay.
Library and binary test configurations MUST retain their production roles.
Missing roots, test-looking names, and conflicting production source roles MUST remain blocking.
Test-only observations MUST remain covered and explicitly disposed as non-production evidence.
They MUST NOT discharge production architecture obligations. Shared production observations MUST NOT be hidden.
The full selected lint and compilation scope MUST remain enabled.
Production evidence MUST retain valid references after partitioning; missing test-only subjects MUST NOT become skipped production checks.
The receipt MUST bind its test-only disposition and the unchanged full-input identities. Artifact replay MUST compare the complete disposition.
No-std evidence MUST come from the active compiler-parsed crate attribute and retain each observing unit's membership.

#### Scenario: Independent tests coexist with production units

- GIVEN explicit integration-test source roles and complete compiler-bound production and test observations
- WHEN the gate evaluates production architecture
- THEN integration tests MUST remain covered with separate dispositions and production findings MUST remain visible

#### Scenario: A production unit imports test-labelled source

- GIVEN a production library, binary, or their test configuration imports test-labelled source
- WHEN ownership is checked
- THEN the source-role conflict MUST block regardless of filename or unit ordering

#### Scenario: Test evidence attempts to satisfy production obligations

- GIVEN test-only witnesses, ports, no-std facts, or unknowns and shared production observations
- WHEN architecture evaluation and artifact replay run
- THEN test-only evidence MUST NOT satisfy production obligations or hide required production unknowns

#### Scenario: A resealed receipt changes its test-only disposition

- GIVEN unchanged compiler inputs and a changed disposition with refreshed receipt, phase-status, and provenance hashes
- WHEN the full artifact bundle is replayed
- THEN reevaluation MUST reject the changed disposition even if receipt self-verification succeeds

#### Scenario: A production import uses a parent-relative path or source alias

- GIVEN test-labelled source imported through a parent-relative path or source symlink
- WHEN source ownership is established
- THEN physical workspace ownership MUST remain visible or collection MUST reject before emitting an accepted architecture result

#### Scenario: Crate configuration differs between production and test observations

- GIVEN active, inactive, library-test-disabled, or integration-only no-std attributes
- WHEN compiler configuration evidence is collected
- THEN only units with the active parsed crate attribute MUST receive no-std facts, and test-only facts MUST NOT satisfy production obligations
