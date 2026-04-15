## ADDED Requirements

### Requirement: Standardized Diagnostic Structure

Every lint diagnostic MUST include:

1. A one-line warning message describing the violation
2. A help note in the exact form `Tiger Style (<principle>): <guidance>`;
   for lints with a concrete approved pattern, `<guidance>` MUST include an
   appended `Example: <approved pattern>` clause in the same help note
3. A note in the exact form `see Tiger Style guide: <section>`

#### Scenario: Developer sees lint for the first time

- GIVEN a developer runs tigerstyle and encounters `ambient_clock`
- WHEN they read the diagnostic
- THEN they see the principle name, the approved pattern, and where to
  read more without opening a separate document

#### Scenario: Diagnostic wording stays consistent across lints

- GIVEN two different tigerstyle lint diagnostics
- WHEN a developer compares their help and note lines
- THEN both diagnostics use the same `Tiger Style (...)` help prefix and
  the same `see Tiger Style guide: ...` note prefix

### Requirement: Lint Groups By Spec Domain

The project MUST register lint groups matching spec domains:

- `tigerstyle::safety`
- `tigerstyle::naming`
- `tigerstyle::structure`
- `tigerstyle::purity`
- `tigerstyle::concurrency`

Consumers MUST be able to use group-level `#[allow]` and `#[deny]`
attributes when compiling on the supported nightly toolchain and opting into
`#![feature(register_tool)]` plus `#![register_tool(tigerstyle)]`.

#### Scenario: Suppressing a domain during phased rollout

- GIVEN a consumer in starter profile compiling on the supported nightly
  toolchain with `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN they add `#![allow(tigerstyle::safety)]` to a crate root
- THEN all safety-domain lints are suppressed for that crate

#### Scenario: Escalating one domain during cleanup

- GIVEN a consumer wants stricter naming cleanup and is compiling on the
  supported nightly toolchain with `#![feature(register_tool)]` and
  `#![register_tool(tigerstyle)]`
- WHEN they add `#![deny(tigerstyle::naming)]` to a crate root
- THEN naming-domain diagnostics in that crate become hard errors
