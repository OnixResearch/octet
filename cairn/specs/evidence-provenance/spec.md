# Evidence Provenance Specification

## Purpose

Defines the `evidence-provenance` capability.

## Requirements

### Requirement: Provenance record

r[octet.evidence_provenance.record] Octet evidence summaries that claim requirement or policy coverage SHOULD include a provenance record with source ID, checker ID, evidence tier, artifact locator, BLAKE3 content hash, and checked scope.

#### Scenario: Provenance links source to artifact

r[octet.evidence_provenance.record.links]

- GIVEN an evidence artifact claims support for a requirement, policy, lint, or transcript assertion
- WHEN the artifact summary is emitted
- THEN the summary SHOULD include a source ID, checker ID, evidence tier, artifact locator, BLAKE3 content hash, and checked scope.
- AND a reviewer MUST be able to identify the originating requirement or policy from the record.

### Requirement: Provenance validation fails closed

r[octet.evidence_provenance.fail_closed] Provenance validation MUST fail closed when required source identity, checker identity, or content identity is missing or inconsistent.

#### Scenario: Missing source ID fails

r[octet.evidence_provenance.fail_closed.missing_source]

- GIVEN a provenance record omits the source ID
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the missing source ID field.

#### Scenario: Unknown checker ID fails

r[octet.evidence_provenance.fail_closed.unknown_checker]

- GIVEN a provenance record names a checker ID that is not known to the registry
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the unknown checker ID.

#### Scenario: Hash mismatch fails

r[octet.evidence_provenance.fail_closed.hash_mismatch]

- GIVEN a provenance record contains a BLAKE3 hash that does not match the referenced deterministic artifact
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the artifact locator.

### Requirement: Provenance registry ownership

r[octet.evidence_provenance.registry] The provenance record family MUST be registered in `standards/policies/registry.ncl` under the correct ownership classification.

#### Scenario: Registry covers provenance records

r[octet.evidence_provenance.registry.covered]

- GIVEN Octet emits provenance records after execution
- WHEN artifact manifest freshness validation runs
- THEN the provenance family MUST be covered as Rust-derived unless an explicit non-candidate rationale applies.

### Requirement: Cross-gate provenance coverage

r[octet.evidence_provenance.cross_gate_coverage] Octet SHOULD use the same provenance record model for deterministic evidence emitted by runner artifact bundles, standards policy checks, fingerprint review manifests, transcript assertions, and generated policy artifacts.

#### Scenario: Release chain names every producer

r[octet.evidence_provenance.cross_gate_coverage.release_chain]

- GIVEN a release operator gathers Octet runner, standards, policy, fingerprint, and transcript artifacts
- WHEN provenance records are emitted or verified for those artifacts
- THEN each deterministic artifact SHOULD have a record with a source ID, checker ID, evidence tier, artifact locator, BLAKE3 hash, and checked scope.
- AND the evidence tier MUST match the producer's evidence type rather than promoting validation or review artifacts to formal proof.

### Requirement: Provenance producer registry

r[octet.evidence_provenance.producer_registry] Provenance validation MUST use a deterministic registry of known checker IDs and source ID families for Octet evidence producers.

#### Scenario: Known producers pass

r[octet.evidence_provenance.producer_registry.known]

- GIVEN provenance records name registered Octet checker IDs and accepted source ID families
- WHEN producer registry validation runs
- THEN validation MUST accept those producer identities.

#### Scenario: Unknown producer fails

r[octet.evidence_provenance.producer_registry.unknown]

- GIVEN a provenance record names an unknown checker ID or unsupported source ID family
- WHEN producer registry validation runs
- THEN validation MUST fail with a diagnostic naming the unknown producer field.

### Requirement: Parent artifact chains

r[octet.evidence_provenance.parent_chain] Provenance records for derived evidence SHOULD identify parent artifacts when the derived artifact summarizes, bundles, scans, attests to, or gates earlier Octet artifacts.

#### Scenario: Derived artifact links parents

r[octet.evidence_provenance.parent_chain.links]

- GIVEN a policy bundle, release smoke receipt, SBOM scan receipt, release supply-chain receipt, or evidence summary is derived from earlier Octet artifacts
- WHEN its provenance record is emitted
- THEN the record SHOULD list parent artifact locators or identities for the artifacts it summarizes.

#### Scenario: Invalid parent chain fails

r[octet.evidence_provenance.parent_chain.invalid]

- GIVEN a provenance record references a missing parent artifact or a set of records forms a parent cycle
- WHEN parent chain validation runs
- THEN validation MUST fail with a diagnostic naming the missing parent or cycle.

### Requirement: Valence provenance export
r[octet.evidence_provenance.valence_export] Octet SHOULD provide an optional export that normalizes verified Octet provenance records, proof cards, receipts, parent links, checker IDs, evidence tiers, and BLAKE3 artifact identities into adapter-ready Valence provenance input.

#### Scenario: Export is written after artifact verification
r[octet.evidence_provenance.valence_export.after_verify]
- GIVEN an Octet artifact directory passes artifact verification
- WHEN the operator requests a Valence provenance export
- THEN Octet SHOULD write a deterministic export containing the verified provenance records, proof-card links, receipt identities, parent artifacts, and diagnostics.

#### Scenario: Export is not a release claim
r[octet.evidence_provenance.valence_export.not_release_claim]
- GIVEN Octet writes a Valence provenance export
- WHEN the export is inspected
- THEN it MUST state or encode that the export is adapter input and does not by itself prove release eligibility, whole-program correctness, or Mantle bundle inclusion.

### Requirement: Valence export fail-closed behavior
r[octet.evidence_provenance.valence_export.fail_closed] Octet MUST fail closed instead of writing a Valence provenance export when Octet-owned artifact verification or linkage validation fails.

#### Scenario: Stale provenance hash blocks export
r[octet.evidence_provenance.valence_export.fail_closed.stale_hash]
- GIVEN a provenance record's BLAKE3 digest does not match the referenced artifact bytes
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report the stale artifact identity.

#### Scenario: Missing parent blocks export
r[octet.evidence_provenance.valence_export.fail_closed.missing_parent]
- GIVEN a provenance record or proof card references a parent artifact that is absent from the artifact directory
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report the missing parent artifact.

#### Scenario: Unknown checker blocks export
r[octet.evidence_provenance.valence_export.fail_closed.unknown_checker]
- GIVEN a provenance record names a checker ID that is not registered in Octet's deterministic producer registry
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report the unknown checker ID.

### Requirement: Valence provenance export CLI
r[octet.evidence_provenance.valence_export.cli] Octet SHOULD provide an operator-facing command or artifact-verifier option that writes a Valence provenance export only after the referenced artifact directory verifies successfully.

#### Scenario: CLI writes export after verification
r[octet.evidence_provenance.valence_export.cli.write]
- GIVEN an Octet artifact directory passes artifact verification
- WHEN the operator runs the Valence provenance export command or verifier flag with an output path
- THEN Octet SHOULD write the deterministic export to that path.

#### Scenario: CLI reports linkage diagnostics
r[octet.evidence_provenance.valence_export.cli.diagnostics]
- GIVEN Octet cannot export because of stale hashes, missing parents, manual-review-only evidence, or proof-card linkage failures
- WHEN the operator requests the export
- THEN Octet SHOULD report deterministic diagnostics naming the blocking evidence.

### Requirement: Evidence tier preservation in Valence export
r[octet.evidence_provenance.valence_export.tier_preservation] Octet MUST preserve evidence tiers in Valence provenance exports and MUST NOT promote validation, review, or imported evidence to `formal_proof` without an Octet-owned verifier receipt.

#### Scenario: Imported Trellis remains manual review
r[octet.evidence_provenance.valence_export.tier_preservation.trellis_import]
- GIVEN an imported Trellis proof bundle has BLAKE3 source/proof identities but has not been rerun with an Octet-owned verifier receipt
- WHEN Octet exports Valence provenance input
- THEN the exported record MUST remain external or `manual_review_required` rather than `formal_proof`.

#### Scenario: Validated proof receipt may export formal proof
r[octet.evidence_provenance.valence_export.tier_preservation.formal_proof]
- GIVEN a proof rail has an Octet-owned validated verifier receipt and matching proof package/artifact identities
- WHEN Octet exports Valence provenance input
- THEN the exported record MAY carry `formal_proof` for the scoped property and checked source/proof binding.

### Requirement: Valence export boundary documentation
r[octet.evidence_provenance.valence_export.boundary] Octet documentation MUST describe the Valence provenance export as an optional adapter input consumed by Valence stack verification, with Mantle receiving only the final sidecar as opaque external evidence.

#### Scenario: Documentation preserves dependency direction
r[octet.evidence_provenance.valence_export.boundary.dependency_direction]
- GIVEN an operator reads Octet's provenance export documentation
- WHEN the stack integration flow is described
- THEN the documentation MUST show Octet exports verified artifact facts, Valence verifies stack semantics, and Mantle bundles the final sidecar opaquely without parsing Octet or Valence semantics.
