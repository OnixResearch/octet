# Valence Project Extraction Specification

## Purpose

Completes the accepted standalone Valence cutover by pinning one hardened canonical source and eliminating ambiguous duplicate package identity.

## Requirements

### Requirement: Hardened standalone Valence is an explicit prerequisite
r[octet.valence_project_extraction.phase5.hardened_upstream_dependency] Octet MUST consume a standalone Valence revision that includes the archived corrected Preserves integrity and typed claim-boundary profile before completing the semantic cutover.

#### Scenario: Vulnerable upstream revision is rejected
- GIVEN a standalone Valence revision lacks the accepted corrected-profile archive receipt or manifest identity
- WHEN Octet validates cutover readiness
- THEN cutover MUST fail with a deterministic upstream-profile diagnostic.

### Requirement: Standalone Valence source is pinned exactly
r[octet.valence_project_extraction.phase5.exact_source_pin] Octet MUST bind release and compatibility evidence to an immutable standalone Valence source coordinate and exact revision, and the dependency manifest and lockfile MUST agree on that identity.

#### Scenario: Exact source and lock identity pass
- GIVEN the dependency manifest and lockfile name the same standalone Valence source and immutable revision
- WHEN dependency validation runs
- THEN the source-pin contribution MUST pass and preserve the source identity in the cutover receipt.

#### Scenario: Floating or drifting source fails
- GIVEN the dependency uses only a branch, tag, floating SSH reference, local override, or a lockfile revision different from the reviewed manifest
- WHEN release cutover validation runs
- THEN validation MUST fail before compatibility evidence is accepted.

### Requirement: Standalone dependency manifest remains complete
r[octet.valence_project_extraction.phase5.standalone_dependency_manifest] Octet MUST validate a dependency manifest identifying the standalone source, exact revision, replaced or shimmed Octet modules, compatibility disposition, fixture identity, and required non-claims.

#### Scenario: Missing migration row fails closed
- GIVEN an Octet-hosted Valence module has no canonical, shimmed, retired, or bounded-exception disposition
- WHEN manifest validation runs
- THEN validation MUST fail with a missing-row diagnostic.

### Requirement: Canonical package identity is unique
r[octet.valence_project_extraction.phase5.unique_package_identity] Octet MUST ensure that materially different implementations cannot share the canonical `valence-core` package name and version in its workspace or resolved dependency graph.

#### Scenario: Hosted package is retired or renamed
- GIVEN standalone Valence supplies canonical `valence-core`
- WHEN the Octet workspace is resolved
- THEN the prior Octet-hosted implementation MUST be absent or use a distinct legacy package identity that cannot be selected as canonical Valence.

#### Scenario: Duplicate canonical identity fails
- GIVEN two different source identities resolve under the same canonical `valence-core` package name and version
- WHEN package-graph validation runs
- THEN validation MUST fail and name both source identities.

### Requirement: Compatibility shims contain no semantic fork
r[octet.valence_project_extraction.phase5.compatibility_shim] Octet MAY retain explicit compatibility shims, but shims MUST delegate Valence-owned pure semantics to standalone Valence and MUST NOT retain an independently evolving canonicalization, claim-policy, or provenance implementation.

#### Scenario: Shell delegation preserves ownership
r[octet.valence_project_extraction.phase5.shell_consumes_standalone]
- GIVEN an Octet command requires a Valence-owned pure operation
- WHEN the command runs
- THEN Octet MUST retain shell effects while delegating the semantic operation to standalone Valence or a non-semantic adapter.

### Requirement: Selected compatibility fixtures pass and drift fails
r[octet.valence_project_extraction.phase5.compatibility] Octet MUST verify declared Function Object, evidence-index, and evidence-codebase-index compatibility before retiring the hosted implementation.

#### Scenario: Positive compatibility corpus passes
r[octet.valence_project_extraction.phase5.compatibility.positive]
- GIVEN reviewed compatibility fixtures and the pinned standalone dependency
- WHEN both accepted compatibility paths are evaluated
- THEN expected schema strings, canonical hashes, outputs, and bounded non-claims MUST match.

#### Scenario: Stale or unsupported compatibility fails
r[octet.valence_project_extraction.phase5.compatibility.negative]
- GIVEN the source revision is wrong, hosted code drifts, a module row is missing, non-claims weaken, lock identity drifts, package identity duplicates, or a call falls back to hosted semantics
- WHEN cutover validation runs
- THEN validation MUST fail with a deterministic diagnostic naming the failure class.

### Requirement: Consumer migration names the canonical target
r[octet.valence_project_extraction.phase5.consumer_migration_docs] Octet MUST document migration from the Octet-hosted package to the exact standalone Valence dependency and identify the compatibility command and bounded non-claims.

#### Scenario: Aspen migration is actionable
- GIVEN Aspen currently resolves Octet's hosted `valence-core`
- WHEN maintainers read the migration guidance
- THEN the guidance MUST name the canonical source, pin policy, compatibility evidence, and removal of the ambiguous hosted package identity.

### Requirement: Cutover validation remains bounded
r[octet.valence_project_extraction.phase5.cutover_validation] Cutover validation MUST prove selected source ownership, unique package identity, dependency pinning, and declared fixture compatibility only.

#### Scenario: Cutover report avoids overclaims
- GIVEN cutover validation passes
- WHEN the report is reviewed
- THEN it MUST NOT claim arbitrary semantic equivalence, behavioral correctness, verifier soundness, external consumer compatibility, or release eligibility.

### Requirement: Flake-check CI is checked in
r[octet.valence_project_extraction.phase5.flake_check_ci] Octet MUST include a checked-in CI workflow for this remediation whose verification command is `nix flake check`.

#### Scenario: CI uses the scoped verification rail
- GIVEN a change is evaluated by checked-in CI
- WHEN the remediation workflow runs
- THEN it MUST execute `nix flake check` without requiring a separate expanded CI command matrix in this change.
