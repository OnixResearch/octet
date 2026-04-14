## ADDED Requirements

### Requirement: Standardized Diagnostic Structure

Every lint diagnostic MUST include:

1. A one-line warning message describing the violation
2. A help note in the form `Tiger Style (<principle>): <guidance>`
3. A note referencing the relevant Tiger Style guide section

#### Scenario: Developer sees lint for the first time

- GIVEN a developer runs tigerstyle and encounters `ambient_clock`
- WHEN they read the diagnostic
- THEN they see the principle name, the approved pattern, and where to
  read more — without opening a separate document

### Requirement: Lint Groups By Spec Domain

The project MUST register lint groups matching spec domains:

- `tigerstyle::safety`
- `tigerstyle::naming`
- `tigerstyle::structure`
- `tigerstyle::purity`
- `tigerstyle::concurrency`

Consumers MUST be able to use group-level `#[allow]` and `#[deny]`
attributes.

#### Scenario: Suppressing a domain during phased rollout

- GIVEN a consumer in starter profile
- WHEN they add `#![allow(tigerstyle::safety)]` to a crate root
- THEN all safety-domain lints are suppressed for that crate
