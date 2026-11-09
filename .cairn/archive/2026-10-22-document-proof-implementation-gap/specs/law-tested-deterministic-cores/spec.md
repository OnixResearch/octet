# Law Tested Deterministic Cores Specification Delta

## MODIFIED Requirements

### Requirement: Law tests complement fixture rails

r[octet.law_tested_deterministic_cores.fixture_complement] Property and law tests MUST complement, not replace, UI fixtures, snapshot tests, CLI golden output, transcript evidence, and artifact verification required for user-visible behavior. Property and law tests over a pure core MUST NOT be presented as evidence that a deployed implementation is correct, and MUST NOT be presented as closing the interleaving, fault-handling, or unverified-extension layers of the proof-to-implementation gap.

#### Scenario: User-visible behavior remains fixture-backed

r[octet.law_tested_deterministic_cores.fixture_complement.user_visible]

- GIVEN a law suite covers a pure classifier, config merge, or evidence transform
- WHEN the behavior produces user-visible diagnostics, CLI output, source-level lint matches, or persisted artifacts
- THEN maintainers SHOULD keep positive and negative fixture or golden-output coverage for the user-visible contract.

#### Scenario: Law tests do not close the proof-to-implementation layers

r[octet.law_tested_deterministic_cores.fixture_complement.proof_implementation_boundary]

- GIVEN a law suite or Hegel property run passes for a pure core
- WHEN the result is reported as implementation assurance
- THEN the report MUST NOT present it as evidence that the deployed implementation is correct.
- AND the report MUST point to the documented proof-to-implementation gap instead.
