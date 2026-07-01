# Verus Proof Rails Specification

## Purpose

Defines the `verus-proof-rails` capability.

## Requirements

### Requirement: Verus proof-rail contract

r[octet.verus_proof_rails.contract] Octet SHOULD support Verus as an opt-in formal-proof rail that is scoped to declared source paths, proof paths, verifier identity, pass signal, and trust boundaries.

#### Scenario: Pinned Verus rail is declared

r[octet.verus_proof_rails.contract.declared]

- GIVEN a policy declares a Verus proof rail
- WHEN Octet exports or validates the policy registry
- THEN the rail MUST declare `backend = "verus"`, a deterministic command, source paths, proof paths, an exact pass signal, and trust-boundary notes.
- AND the rail MUST NOT rely on an ambient unpinned `verus` binary as verifier identity for `formal_proof` evidence.

#### Scenario: Proof claim remains scoped

r[octet.verus_proof_rails.contract.scoped]

- GIVEN a Verus proof rail passes
- WHEN Octet renders proof evidence or documentation for that rail
- THEN the evidence MUST name the checked source scope, proof artifact identity, verifier identity, and covered property or policy ID.
- AND the evidence MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, or full safety without separate evidence.

### Requirement: Persistent Verus proof receipts

r[octet.verus_proof_rails.receipts] Octet MUST persist proof receipts for executed Verus rails when the caller requests proof artifacts, and those receipts MUST be validated before they can support `formal_proof` provenance.

#### Scenario: Matching proof receipt passes

r[octet.verus_proof_rails.receipts.matching]

- GIVEN a Verus proof rail has a persisted receipt containing the configured pass signal
- AND the receipt contains matching BLAKE3 command and source bindings when the rail enables command/source binding
- AND the receipt contains configured verifier, provenance, and trust-boundary lines
- WHEN proof receipt validation runs
- THEN validation MUST accept the receipt.
- AND Octet MAY emit `formal_proof` provenance for the proof artifact.

#### Scenario: Stale or incomplete proof receipt fails

r[octet.verus_proof_rails.receipts.fail_closed]

- GIVEN a Verus proof receipt is missing the configured pass signal, has a stale source BLAKE3 binding, has wrong verifier identity, omits a required trust-boundary line, or omits proof artifact identity
- WHEN proof receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the offending receipt field.
- AND Octet MUST NOT emit `formal_proof` provenance for that receipt.

#### Scenario: Proof artifact verifier is read-only

r[octet.verus_proof_rails.receipts.artifact_verify]

- GIVEN an operator has a saved proof artifact directory
- WHEN `cargo octet artifact verify` validates proof receipts and provenance
- THEN the verifier MUST recompute BLAKE3 identities and validate provenance coverage without mutating the artifact directory.
- AND malformed, missing, stale, or unmatched proof artifacts MUST fail closed.

### Requirement: Proof command surface

r[octet.verus_proof_rails.prove_command] Octet SHOULD expose a `cargo octet prove` command that runs selected proof rails and writes reviewable proof artifacts without changing ordinary lint-check behavior.

#### Scenario: Prove command writes scoped artifacts

r[octet.verus_proof_rails.prove_command.artifacts]

- GIVEN a workspace declares a selected Verus proof rail
- WHEN the operator runs `cargo octet prove` with an artifact directory
- THEN Octet SHOULD run the pinned verifier command, write a proof receipt, write provenance with `evidence_tier = formal_proof` only for validated proof receipts, and report the checked scope.
- AND the command SHOULD return a non-zero status when any selected proof rail fails validation.

#### Scenario: Starter lint path remains unaffected

r[octet.verus_proof_rails.prove_command.starter_unaffected]

- GIVEN a consumer runs ordinary `cargo octet check` under a starter profile with no selected proof rails
- WHEN the check completes
- THEN Octet MUST NOT require Verus to be installed or executed.
- AND the absence of Verus artifacts MUST NOT be rendered as a formal-proof failure for that starter check.

### Requirement: Verus proof registry ownership

r[octet.verus_proof_rails.registry] Verus proof receipt and verifier identity families MUST be registered in `standards/policies/registry.ncl` with Rust-derived ownership or explicit non-candidate rationale.

#### Scenario: Registry covers proof receipt family

r[octet.verus_proof_rails.registry.covered]

- GIVEN Octet emits Verus proof receipts after execution
- WHEN config/receipt family validation runs
- THEN the Verus proof receipt family MUST be registered as Rust-derived.
- AND generated policy artifacts MUST include the proof-rail fields needed by downstream receipt validators.

#### Scenario: Raw verifier internals remain excluded

r[octet.verus_proof_rails.registry.non_candidate]

- GIVEN verifier internals, raw logs, raw environment state, or secrets are encountered during proof execution
- WHEN registry ownership is reviewed
- THEN those raw surfaces MUST be excluded from Nickel-owned config and receipts except as bounded paths, labels, digests, allowlists, or explicit non-candidate rationale.

### Requirement: Verified-module proof coverage

r[octet.verus_proof_rails.verified_module_coverage] Octet verification profiles MUST make unbacked `verified/` module or formal-proof claims explicit instead of silently treating purity lint success as proof.

#### Scenario: Covered verified module passes

r[octet.verus_proof_rails.verified_module_coverage.covered]

- GIVEN a source file under `verified/` is selected by a verification profile
- AND the policy registry declares a matching Verus proof rail for that source file
- WHEN verification-profile checks run
- THEN Octet MUST accept the verified-module coverage obligation only after the proof receipt validates.

#### Scenario: Unbacked verified claim is manual review or failure

r[octet.verus_proof_rails.verified_module_coverage.unbacked]

- GIVEN a source file, document, or artifact claims formal verification for a module but no matching proof rail or validated proof receipt exists
- WHEN verification-profile checks run
- THEN Octet MUST either fail the selected verification gate or emit `manual_review_required` provenance with a human-action message, according to the selected profile.
- AND Octet MUST NOT emit `formal_proof` provenance for the unbacked claim.

### Requirement: Production Verus verifier identity

r[octet.verus_proof_rails.verifier_identity] Octet MUST distinguish production Verus verifier identity from fixture-only verifier identity before emitting promotion-grade `formal_proof` evidence.

#### Scenario: Production verifier identity is complete

r[octet.verus_proof_rails.verifier_identity.production]

- GIVEN a Verus proof rail is eligible for production `formal_proof` provenance
- WHEN `cargo octet prove` records verifier identity for that rail
- THEN the identity MUST include the pinned verifier label, verifier binary digest, solver identity, Rust toolchain identity, and Nix closure or closure-graph digest.
- AND the identity MUST be recorded in the proof receipt and provenance record.

#### Scenario: Fixture verifier remains bounded

r[octet.verus_proof_rails.verifier_identity.fixture]

- GIVEN a proof run uses the fixture verifier wrapper
- WHEN Octet writes proof artifacts for that run
- THEN the receipt and summary MUST label the evidence as fixture-only.
- AND Octet MUST NOT promote that receipt as production `formal_proof` provenance unless the checked scope explicitly declares fixture-only proof plumbing.

### Requirement: Structured Verus proof receipts

r[octet.verus_proof_rails.structured_receipts] Octet MUST persist Verus proof receipts as structured Rust-owned records with deterministic validation before those receipts can support `formal_proof` provenance.

#### Scenario: Valid structured receipt passes

r[octet.verus_proof_rails.structured_receipts.valid]

- GIVEN a Verus proof rail has a structured receipt with schema version, policy ID, rail index, verifier identity, command identity, source bindings, proof bindings, trust boundaries, checked scope, pass signal, and proof artifact identity
- WHEN proof receipt validation runs
- THEN validation MUST accept the receipt.
- AND Octet MAY emit `formal_proof` provenance for the matching proof artifact.

#### Scenario: Malformed or stale structured receipt fails closed

r[octet.verus_proof_rails.structured_receipts.fail_closed]

- GIVEN a structured receipt omits a required field, uses an unsupported schema version, has stale source or proof bindings, has wrong verifier identity, omits proof artifact identity, or records fixture evidence as production evidence
- WHEN proof receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field.
- AND Octet MUST NOT emit `formal_proof` provenance for that receipt.

#### Scenario: Receipt contract is registered

r[octet.verus_proof_rails.structured_receipts.contract]

- GIVEN Octet emits structured Verus proof receipts from Rust code
- WHEN policy freshness validation runs
- THEN the Verus proof receipt contract MUST be registered as a Rust-derived receipt family.
- AND the committed Nickel contract MUST match the Rust-owned receipt schema source.

### Requirement: Safe Verus proof command execution

r[octet.verus_proof_rails.safe_execution] Octet MUST execute proof rails through a deterministic command model instead of an unchecked shell command string.

#### Scenario: Proof command uses argv and bounded environment

r[octet.verus_proof_rails.safe_execution.argv]

- GIVEN a selected Verus proof rail declares an executable and arguments
- WHEN `cargo octet prove` runs the rail
- THEN the CLI shell MUST execute the pinned verifier with explicit argv, a deterministic working directory, an allowlisted environment, a configured timeout, and bounded stdout/stderr artifact capture.
- AND the pure validation core MUST receive only typed command results and in-memory receipt inputs.

#### Scenario: Proof command failures fail closed

r[octet.verus_proof_rails.safe_execution.failures]

- GIVEN proof command spawning fails, exits unsuccessfully, times out, exceeds bounded log capture, or resolves to an unpinned verifier
- WHEN `cargo octet prove` reports the run
- THEN the proof row MUST fail with a deterministic diagnostic naming the command failure class.
- AND Octet MUST NOT emit `formal_proof` provenance for that failed run.

#### Scenario: Starter lint path remains independent

r[octet.verus_proof_rails.safe_execution.starter_independent]

- GIVEN a consumer runs ordinary `cargo octet check` under a starter profile
- WHEN no proof rails are selected
- THEN Octet MUST NOT require Verus, solver binaries, proof manifests, or proof artifacts.
- AND missing proof tooling MUST NOT affect the starter lint result.

### Requirement: Normalized proof rail path matching

r[octet.verus_proof_rails.path_matching] Octet MUST bind proof rails to normalized repo-relative source and proof paths using exact or glob semantics rather than substring matching.

#### Scenario: Exact and glob matches are deterministic

r[octet.verus_proof_rails.path_matching.deterministic]

- GIVEN a proof rail declares exact source paths, exact proof paths, or explicit glob patterns
- WHEN rail selection and receipt validation run
- THEN Octet MUST normalize candidate paths to repo-relative form before matching.
- AND Octet MUST bind each matched path deterministically in sorted order for receipt hashing and diagnostics.

#### Scenario: Ambiguous or overbroad matches fail closed

r[octet.verus_proof_rails.path_matching.ambiguous]

- GIVEN a rail path pattern is empty, matches no selected source, relies only on substring matching, or matches multiple paths where the rail requires a single artifact
- WHEN rail selection or verified-module coverage runs
- THEN Octet MUST report a deterministic diagnostic naming the path pattern.
- AND Octet MUST NOT treat the rail as proof coverage for the affected source.

### Requirement: Nickel proof rail manifest

r[octet.verus_proof_rails.rail_manifest] Human-authored Verus proof rail declarations SHOULD be maintained as typed Nickel and exported as deterministic JSON for runtime Rust consumers.

#### Scenario: Valid Nickel manifest exports deterministic rails

r[octet.verus_proof_rails.rail_manifest.valid]

- GIVEN a maintainer declares Verus proof rails in the Nickel manifest
- WHEN the manifest export command runs
- THEN the exported JSON MUST include backend, command argv, source paths, proof paths, verifier identity requirements, pass signal, trust boundaries, and receipt binding policy.
- AND repeated exports from the same manifest MUST produce byte-identical JSON.

#### Scenario: Invalid Nickel manifest fails before execution

r[octet.verus_proof_rails.rail_manifest.invalid]

- GIVEN a manifest omits required rail fields, declares an unsupported backend, uses empty source paths, or provides malformed trust boundaries
- WHEN manifest validation runs
- THEN validation MUST fail before `cargo octet prove` executes any verifier.
- AND the diagnostic MUST name the rejected manifest field.

### Requirement: Verus proof operator feedback

r[octet.verus_proof_rails.operator_feedback] Octet SHOULD provide list and explain surfaces for proof rails without broadening formal-proof claims.

#### Scenario: Selected rails can be listed

r[octet.verus_proof_rails.operator_feedback.list]

- GIVEN a workspace has exported Verus proof rails
- WHEN an operator runs the proof rail listing surface
- THEN Octet SHOULD show each selected rail's policy ID, backend, source scope, proof path, verifier label, and fixture-or-production mode.
- AND the output MUST include scoped non-claim wording for formal proof evidence.

#### Scenario: Unbacked claims can be explained

r[octet.verus_proof_rails.operator_feedback.explain]

- GIVEN a verified module or formal-proof claim lacks a matching validated receipt
- WHEN Octet reports or explains the claim
- THEN the output SHOULD identify the unmatched source, missing or stale rail field, required command, and remediation.
- AND Octet MUST NOT emit `formal_proof` provenance for the unbacked claim.
