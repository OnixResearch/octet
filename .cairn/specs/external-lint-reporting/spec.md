# External Lint Reporting Specification

## Purpose

Defines the `external-lint-reporting` capability.

## Requirements

### Requirement: Clippy co-reporting rail

r[octet.external_lint.clippy_co_reporting] Octet SHOULD provide an optional Clippy co-reporting rail that can run Clippy or import Clippy JSON while preserving Clippy's diagnostic identity.

#### Scenario: Clippy findings are preserved

r[octet.external_lint.clippy_co_reporting.preserved]

- GIVEN Clippy emits a lint finding with a lint name, level, message, span, and suggestion metadata
- WHEN Octet imports or co-runs Clippy
- THEN Octet SHOULD preserve those fields in the external-lint sidecar.
- AND Octet MUST NOT relabel the finding as an Octet lint ID.

#### Scenario: Clean Clippy run joins artifact bundle

r[octet.external_lint.clippy_co_reporting.clean]

- GIVEN a consumer opts into Clippy co-reporting and Clippy exits successfully
- WHEN Octet writes an artifact directory
- THEN the artifact directory SHOULD include a deterministic Clippy summary or receipt.
- AND artifact verification SHOULD cover that receipt when present.

### Requirement: External lint severity mapping

r[octet.external_lint.severity_mapping] External lint reporting MUST preserve source checker severity while making Octet gate behavior explicit.

#### Scenario: Source and gate levels are distinct

r[octet.external_lint.severity_mapping.distinct]

- GIVEN a Clippy finding has a source lint level and Octet applies a separate gate policy
- WHEN Octet renders the finding
- THEN the report MUST distinguish the Clippy source level from the Octet gate outcome.

#### Scenario: Import failure fails deterministically

r[octet.external_lint.severity_mapping.import_failure]

- GIVEN a Clippy JSON report is malformed or does not match the expected message format
- WHEN Octet imports the report
- THEN Octet MUST fail with a diagnostic naming the malformed input and field.

### Requirement: External lint provenance

r[octet.external_lint.provenance] External Clippy evidence SHOULD use `static_lint` provenance with a registered Clippy checker identity.

#### Scenario: Provenance identifies Clippy

r[octet.external_lint.provenance.checker]

- GIVEN Octet emits a Clippy co-reporting artifact
- WHEN provenance is written
- THEN the provenance record SHOULD name `clippy` or the registered Clippy checker ID, evidence tier `static_lint`, artifact locator, BLAKE3 content hash, and checked scope.

#### Scenario: Hosted scanners stay optional

r[octet.external_lint.provenance.no_hosted_requirement]

- GIVEN Clippy co-reporting is enabled
- WHEN the rail runs in a controlled-deployment environment
- THEN it MUST NOT require SonarQube, CodeQL, Semgrep, GitHub code scanning, or any hosted service.

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

- GIVEN an operator supplies a known profile ID to Clippy run mode, with either the shipped embedded catalog or an explicit checked catalog
- WHEN Octet prepares the tool invocation
- THEN it MUST execute quiet workspace/all-target Clippy JSON output with only the profile's explicit lint-level flags appended after Cargo arguments.
- AND a shipped profile MUST be selectable without a workspace-relative Octet catalog path.
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

#### Scenario: Policy owner dogfoods production safety

r[octet.external_lint.clippy_restriction_profiles.verification.owner_dogfood]

- GIVEN Octet's generated catalog contains the `production-safety` profile
- WHEN the repo-owned flake Clippy check constructs its lint arguments
- THEN it MUST derive the profile's exact lint level and lint IDs from the generated catalog.
- AND the workspace/all-target check MUST fail on any selected restriction finding.
