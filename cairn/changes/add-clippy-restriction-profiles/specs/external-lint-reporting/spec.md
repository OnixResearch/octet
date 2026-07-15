# External Lint Reporting Specification

## Purpose

Adds typed, profile-bound Clippy restriction execution to the existing external lint reporting capability.

## Requirements

### Requirement: Nickel-authored Clippy restriction profile catalog

r[octet.external_lint.clippy_restriction_profiles.catalog] Octet MUST provide a typed Nickel-authored catalog of narrow Clippy restriction profiles with checked-in generated runtime JSON, unique profile IDs, explicit lint levels, explicit supported lint identities, claim scope, and non-claims.

#### Scenario: Valid catalog exports deterministically

r[octet.external_lint.clippy_restriction_profiles.catalog.valid]

- GIVEN the reviewed Nickel catalog declares the supported Clippy restriction profiles
- WHEN Nickel typechecking and deterministic export run
- THEN the generated JSON MUST match the checked-in artifact byte-for-byte.
- AND the catalog MUST include `production-safety`, `canonical-determinism`, and `shell-boundary-inventory`.
- AND no profile MAY enable the blanket `clippy::restriction` group.

#### Scenario: Invalid catalog fails closed

r[octet.external_lint.clippy_restriction_profiles.catalog.invalid]

- GIVEN a profile has an unsupported schema, duplicate or empty profile ID, duplicate or unsupported lint ID, empty lint set, invalid lint level, invalid claim scope, or missing non-claim
- WHEN Nickel or runtime validation runs
- THEN validation MUST fail closed with deterministic diagnostics naming the invalid profile or field.

### Requirement: Profiled Clippy restriction execution

r[octet.external_lint.clippy_restriction_profiles.run] The Clippy co-reporting rail MUST support opt-in execution of a selected checked restriction profile without granting arbitrary command authority.

#### Scenario: Selected profile builds exact command

r[octet.external_lint.clippy_restriction_profiles.run.profiled]

- GIVEN an operator supplies a checked generated catalog and known profile ID to Clippy run mode
- WHEN Octet prepares the tool invocation
- THEN it MUST execute quiet workspace/all-target Clippy JSON output with only the profile's explicit lint-level flags appended after Cargo arguments.
- AND it MUST reject an operator-supplied arbitrary command for that profiled run.

#### Scenario: Unprofiled run remains compatible

r[octet.external_lint.clippy_restriction_profiles.run.compatibility]

- GIVEN an existing operator invokes Clippy co-reporting run mode without a config and profile
- WHEN Octet prepares the command
- THEN the existing default Clippy co-reporting command MUST remain available.
- AND import mode MUST continue preserving external Clippy JSON identities.

### Requirement: Policy-bound Clippy restriction receipt

r[octet.external_lint.clippy_restriction_profiles.receipt] A profiled Clippy run receipt MUST bind the reviewed policy bytes, selected profile, exact lint posture, command, findings, and tool outcome.

#### Scenario: Profile and config identities are bound

r[octet.external_lint.clippy_restriction_profiles.receipt.policy_bound]

- GIVEN a profiled Clippy restriction run writes an evidence-rail receipt
- WHEN receipt validation runs
- THEN the receipt MUST include the selected profile ID, selected-profile BLAKE3 identity, full config BLAKE3 identity, lint level, exact lint IDs, and exact reconstructed command argv.
- AND artifact verification MUST reject missing, malformed, mismatched, unsupported, or command-drifted profile fields.

#### Scenario: Tool failure cannot appear clean

r[octet.external_lint.clippy_restriction_profiles.receipt.tool_failure]

- GIVEN Clippy or Cargo exits nonzero because of denied findings, compilation failure, or integration failure
- WHEN Octet evaluates the run
- THEN the receipt MUST fail with the tool exit code preserved.
- AND non-JSON stderr MUST NOT be parsed as Clippy JSON.

### Requirement: Bounded Clippy restriction claims and verification

r[octet.external_lint.clippy_restriction_profiles.boundaries] Clippy restriction profiles MUST remain complementary opt-in static-lint evidence and MUST NOT replace Octet semantic lint authority or imply behavioral correctness.

#### Scenario: Blanket restriction group remains forbidden

r[octet.external_lint.clippy_restriction_profiles.boundaries.no_blanket]

- GIVEN an operator reviews the shipped profiles and documentation
- WHEN profile scope and claims are described
- THEN Octet MUST state that the entire `clippy::restriction` group is unsupported.
- AND Octet MUST NOT claim panic freedom, memory safety, runtime determinism, compiler correctness, formal proof, whole-program correctness, or release eligibility from a passing Clippy profile.

### Requirement: Positive and negative Clippy restriction verification

r[octet.external_lint.clippy_restriction_profiles.verification] The profile implementation MUST include positive and negative tests for catalog validation, command construction, clean and violating Clippy runs, tool failures, policy identity, and receipt tampering.

#### Scenario: Focused verification distinguishes clean and violating workspaces

r[octet.external_lint.clippy_restriction_profiles.verification.focused]

- GIVEN one fixture is clean under a shipped profile and another triggers a selected restriction lint
- WHEN the profiled Clippy evidence rail runs against both fixtures
- THEN the clean fixture MUST produce a passing policy-bound receipt.
- AND the violating fixture MUST produce a failed receipt preserving the Clippy lint identity and nonzero tool exit.
