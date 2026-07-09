# Dependency Trust Specification

## Purpose

Defines the `dependency-trust` capability.

## Requirements

### Requirement: Cargo-vet dependency trust receipts

r[octet.dependency_trust.vet_receipts] Octet SHOULD support optional dependency-trust receipts derived from `cargo-vet` execution or saved `cargo-vet` output.

#### Scenario: Fully covered audit passes

r[octet.dependency_trust.vet_receipts.covered]

- GIVEN `cargo-vet` reports that all configured third-party dependencies satisfy the project's audit criteria or trusted imports
- WHEN Octet records dependency-trust evidence
- THEN Octet SHOULD emit a receipt with successful status, audited package coverage, imports used, command argv, and config root.

#### Scenario: Audit gap requires review

r[octet.dependency_trust.vet_receipts.gap]

- GIVEN `cargo-vet` reports unaudited dependencies, unresolved criteria, or missing imports
- WHEN Octet records dependency-trust evidence
- THEN Octet MUST preserve the gap in the receipt.
- AND the evidence tier SHOULD be `manual_review_required` unless the configured policy explicitly treats the gap as a hard failure.

### Requirement: Dependency trust exceptions

r[octet.dependency_trust.exceptions] Dependency trust evidence MUST make exemptions and unaudited allowances explicit instead of hiding them inside a clean summary.

#### Scenario: Exemption appears in receipt

r[octet.dependency_trust.exceptions.visible]

- GIVEN dependency trust configuration includes an exemption or deferred audit allowance
- WHEN Octet emits a dependency-trust receipt
- THEN the receipt MUST include the package, version or criteria scope, rationale field when available, and review action.

#### Scenario: Silent trust import is rejected

r[octet.dependency_trust.exceptions.silent_import]

- GIVEN a dependency-trust result relies on an imported audit source
- WHEN Octet records the result
- THEN the receipt MUST identify the import source.
- AND validation MUST fail if the import source identity is absent from the receipt.

### Requirement: Dependency trust non-claims

r[octet.dependency_trust.non_claims] Dependency trust receipts MUST state that audit coverage and trust imports do not prove dependencies are benign or vulnerability-free.

#### Scenario: Receipt keeps trust boundary

r[octet.dependency_trust.non_claims.boundary]

- GIVEN a dependency-trust receipt reports successful coverage
- WHEN a reviewer inspects the receipt
- THEN the receipt MUST preserve non-claims for malicious maintainer behavior, undiscovered vulnerabilities, and project-specific business risk.

#### Scenario: Cargo-deny remains separate

r[octet.dependency_trust.non_claims.separate_from_deny]

- GIVEN both dependency-audit and dependency-trust rails are configured
- WHEN Octet renders the evidence summary
- THEN advisory/license policy results and human audit coverage MUST appear as separate evidence entries with separate checker IDs.

### Requirement: Source boundary core contract

r[octet.dependency_trust.source_boundary_core.contract] Octet MUST define pure source-boundary contracts for source kind, revision identity, dirty-state policy, vendored snapshot metadata, registry package metadata, generated artifact metadata, digest inventories, and required non-claims.

#### Scenario: Valid source descriptor validates

r[octet.dependency_trust.source_boundary_core.fixtures.positive]

- GIVEN a source descriptor has a supported source kind, required revision or package identity, selected-path policy, digest inventory, dirty-state disposition, and non-claim boundary
- WHEN source-boundary validation runs
- THEN validation MUST pass and normalize the descriptor identity.

#### Scenario: Invalid source descriptor fails closed

r[octet.dependency_trust.source_boundary_core.fixtures.negative]

- GIVEN a source descriptor has missing revision metadata, disallowed dirty state, path escape, missing digest inventory, stale vendored digest, unknown source kind, missing non-claims, or dependency-trust overclaim
- WHEN source-boundary validation runs
- THEN validation MUST fail with deterministic diagnostics naming the invalid descriptor field.

### Requirement: Vendored snapshots are explicit source boundaries

r[octet.dependency_trust.source_boundary_core.vendored] Octet MUST require vendored source descriptors to bind upstream source identity, revision, selected paths, digest inventory, and local-patch disposition.

#### Scenario: Vendored snapshot binds upstream revision

r[octet.dependency_trust.source_boundary_core.vendored.revision]

- GIVEN a source descriptor identifies a vendored snapshot
- WHEN validation runs
- THEN upstream source identity, upstream revision, selected paths, and digest inventory MUST be present and aligned.

### Requirement: Source-boundary core is pure

r[octet.dependency_trust.source_boundary_core.validation] Source-boundary validation MUST be pure deterministic logic over in-memory descriptors and inventories.

#### Scenario: Shell owns source discovery

r[octet.dependency_trust.source_boundary_core.validation.shell]

- GIVEN Octet gathers source facts from Git, Cargo metadata, local worktrees, vendored directories, generated artifacts, or registries
- WHEN source-boundary validation is invoked
- THEN the shell MUST pass parsed descriptors to the pure core rather than letting the core read files or perform network access.

### Requirement: Source-boundary adapters preserve existing receipts

r[octet.dependency_trust.source_boundary_core.adapters] Octet SHOULD provide adapters for existing cross-repo source acquisition and dependency-trust receipts without changing their public field names or trust boundaries.

#### Scenario: Existing source receipt adapts

r[octet.dependency_trust.source_boundary_core.adapters.receipt]

- GIVEN an existing Octet source acquisition or dependency-trust receipt uses current public source fields
- WHEN an adapter validates the receipt
- THEN the adapter SHOULD preserve those fields while routing reusable source-boundary checks through the shared core.

### Requirement: Source-boundary core remains bounded

r[octet.dependency_trust.source_boundary_core.docs] Passing source-boundary validation MUST NOT prove dependency trust, code correctness, supply-chain safety, release eligibility, verifier soundness, or downstream behavioral correctness.

#### Scenario: Boundary is visible

r[octet.dependency_trust.source_boundary_core.final_validation]

- GIVEN a source-boundary validation report passes
- WHEN an operator reads the supported claim
- THEN the report or documentation MUST state the bounded non-claims and point to focused validation evidence.
