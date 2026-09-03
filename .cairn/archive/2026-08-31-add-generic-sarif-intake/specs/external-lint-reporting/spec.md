# External Lint Reporting Specification Delta

## ADDED Requirements

### Requirement: Generic SARIF intake uses an explicit producer profile

r[octet.external_lint.sarif_intake_profile]

Octet MUST define an opt-in SARIF intake profile that binds the supported SARIF version, producer identities, location policy, severity mapping, resource limits, evidence tier, and non-claims.

#### Scenario: Declared producer profile passes

- GIVEN a profile declares a supported SARIF version, driver identity, rule requirements, local scope, severity mapping, limits, and non-claims
- WHEN Octet validates the profile
- THEN validation MUST produce one deterministic profile identity
- AND the identity MUST bind every normalization and gate-policy input

#### Scenario: Broad or incomplete profile fails

- GIVEN a profile permits unknown versions, ambient producers, remote locations, missing limits, arbitrary severity mapping, or weakened non-claims
- WHEN profile validation runs
- THEN validation MUST fail before report parsing

### Requirement: Generic SARIF intake preserves source findings

r[octet.external_lint.sarif_intake]

Octet MUST import a bounded SARIF `2.1.0` subset without relabeling external rule IDs, messages, source severity, locations, or producer identity as Octet-native lint facts.

#### Scenario: Supported findings normalize deterministically

- GIVEN a local SARIF report contains a declared driver, rules, results, safe artifact locations, regions, messages, and supported levels
- WHEN the pure intake core evaluates the supplied bytes twice
- THEN both normalized finding projections and diagnostic ordering MUST match
- AND each finding MUST retain its source driver, rule ID, level, message, and location

#### Scenario: Malformed or unsupported report fails

- GIVEN a report is malformed, uses an unsupported version, omits required producer or rule identity, or contains unsupported result structure
- WHEN intake validation runs
- THEN validation MUST fail with a deterministic field or structure diagnostic
- AND no successful receipt MUST be published

### Requirement: SARIF intake binds raw and normalized identities

r[octet.external_lint.sarif_intake_identity]

A SARIF intake receipt MUST bind the raw report, selected producer profile, normalized projection, checked scope, findings, gate outcome, diagnostics, and BLAKE3 identities.

#### Scenario: Raw and normalized artifacts verify

- GIVEN a supported report imports successfully
- WHEN artifact validation runs
- THEN validation MUST remeasure the raw report and normalized projection
- AND provenance MUST preserve the external checker identity and `static_lint` evidence tier

#### Scenario: Source and gate severity differ

- GIVEN an external finding has one source level and Octet applies a different gate outcome
- WHEN the receipt is rendered
- THEN source severity and Octet gate outcome MUST remain distinct fields
- AND Octet MUST NOT rewrite the source level

#### Scenario: Stale parent identity fails

- GIVEN the raw report, profile, normalized projection, or parent artifact changes after collection
- WHEN artifact validation runs
- THEN validation MUST fail with a deterministic identity diagnostic

### Requirement: SARIF intake remains local and bounded

r[octet.external_lint.sarif_intake_boundary]

SARIF intake MUST use explicit local artifacts, safe workspace-relative locations, bounded resource policy, and non-proof evidence tiers without requiring hosted scanners or network access.

#### Scenario: Hosted scanner is not required

- GIVEN an operator has a local SARIF report from a declared producer
- WHEN the import rail runs
- THEN Octet MUST complete without contacting CodeQL, Semgrep, SonarQube, GitHub code scanning, or another hosted service

#### Scenario: Unsafe location fails

- GIVEN a result location uses parent traversal, a remote URI, a non-file scheme, an invalid artifact index, or an ambiguous path
- WHEN location validation runs
- THEN validation MUST reject or exclude it only through explicit profile policy
- AND the receipt MUST NOT grant filesystem or network authority

#### Scenario: Claim promotion fails

- GIVEN an imported report requests `formal_proof`, complete vulnerability detection, semantic equivalence, or whole-program correctness
- WHEN Octet validates the receipt
- THEN validation MUST fail with a tier or overclaim diagnostic

### Requirement: SARIF intake verification covers positive and negative paths

r[octet.external_lint.sarif_intake_verification]

The implementation MUST include deterministic positive and negative fixtures for profile validation, parsing, identity preservation, locations, severity, limits, provenance, artifact validation, and hosted-service independence.

#### Scenario: Focused fixture matrix completes

- GIVEN clean, finding, malformed, unsupported, unsafe, stale, duplicate, over-limit, and overclaim fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
