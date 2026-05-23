# Valence Project Extraction Specification

## Purpose

Define Phase 0 requirements for preparing Valence to become a standalone project while preserving Octet's current bounded evidence claims and avoiding premature code movement.

## Requirements

### Requirement: Phase 0 extraction inventory
r[octet.valence_project_extraction.phase0.inventory] The system MUST record a reviewed Phase 0 inventory of candidate Valence-owned surfaces, Octet-retained surfaces, shared handoff surfaces, and current verification entrypoints before any code or repository cutover occurs.

#### Scenario: Inventory records current surfaces
r[octet.valence_project_extraction.phase0.inventory.current]
- GIVEN the current Octet repository contains Valence-oriented code, fixtures, docs, Cairn specs, and Nix gates
- WHEN the Phase 0 extraction package is reviewed
- THEN it provides an inventory artifact that names the observed path families and verification entrypoints.

### Requirement: Extraction boundary remains planning-only
r[octet.valence_project_extraction.phase0.boundary] The system MUST classify extraction surfaces without moving code, renaming crates, changing package identity, creating a standalone repository, or cutting a release during Phase 0.

#### Scenario: Boundary avoids cutover side effects
r[octet.valence_project_extraction.phase0.boundary.planning_only]
- GIVEN Phase 0 is an extraction-readiness planning package
- WHEN the package is validated
- THEN it does not require code movement, crate renames, repository creation, tag movement, or release publication.

### Requirement: Handoff gates are explicit
r[octet.valence_project_extraction.phase0.gates] The system MUST define the named Nix and Cairn gates that a future Valence extraction must preserve before any cutover can be accepted.

#### Scenario: Gate contract names release and aggregate rails
r[octet.valence_project_extraction.phase0.gates.named]
- GIVEN the current Octet repository exposes Valence evidence release and aggregate gates
- WHEN the Phase 0 handoff contract is inspected
- THEN it names the release evidence gate, aggregate object evidence gate, Cairn validation gate, and current evidence-index checker invocation or an explicit compatibility successor.

### Requirement: Extraction plan preserves bounded claims
r[octet.valence_project_extraction.phase0.non_claims] The system MUST preserve the existing bounded non-claim language when describing Valence extraction readiness.

#### Scenario: Non-claims stay explicit
r[octet.valence_project_extraction.phase0.non_claims.explicit]
- GIVEN the current Valence evidence is selected-corpus and selected-fixture replay evidence
- WHEN Phase 0 readiness is described
- THEN it does not claim correctness, semantic equivalence, exhaustive Rust coverage, general HIR/THIR/MIR extraction, public unattended production readiness, cross-machine determinism, or cross-toolchain determinism.

### Requirement: Phase 1 in-repository core crate seam
r[octet.valence_project_extraction.phase1.inrepo_core_crate] The system MUST provide an in-repository `valence-core` workspace crate for pure deterministic Valence evidence logic before any standalone repository cutover.

#### Scenario: Core crate is workspace-owned
r[octet.valence_project_extraction.phase1.inrepo_core_crate.workspace]
- GIVEN Octet remains the repository and CLI shell during Phase 1
- WHEN the workspace manifest is inspected
- THEN `valence-core` is a workspace member and can be tested independently.

### Requirement: Evidence-index core moves behind the Valence crate boundary
r[octet.valence_project_extraction.phase1.evidence_index_core] The system MUST move the deterministic evidence-index render/check core into `valence-core` while preserving the existing generated Markdown contract and fail-closed issue semantics.

#### Scenario: Evidence-index core remains deterministic
r[octet.valence_project_extraction.phase1.evidence_index_core.deterministic]
- GIVEN the evidence-index renderer receives a capture time and payload commit
- WHEN `valence-core` renders and checks the page text
- THEN the roundtrip passes and negative stale/missing/weakened inputs fail closed with typed issue codes.

### Requirement: Cargo Octet remains the imperative shell
r[octet.valence_project_extraction.phase1.cli_shell] The system MUST keep filesystem access, argument parsing, stdout/stderr, and command exit behavior in `cargo-octet` while delegating pure evidence-index behavior to `valence-core`.

#### Scenario: CLI shell delegates to core
r[octet.valence_project_extraction.phase1.cli_shell.delegates]
- GIVEN an operator runs `cargo-octet evidence-index check --page docs/current-head-valence-evidence.md`
- WHEN the command validates the checked-in page
- THEN it emits the existing `valence_evidence_index.valid: PASS` and `valence_evidence_index.issue_count=0` pass lines.

### Requirement: Phase 1 preserves existing Valence gates
r[octet.valence_project_extraction.phase1.gates] The system MUST preserve the named Valence evidence Nix gates and Cairn validation after introducing the in-repo core crate.

#### Scenario: Named gates still pass
r[octet.valence_project_extraction.phase1.gates.named]
- GIVEN the in-repo Valence crate seam has been added
- WHEN focused Rust tests, `valence-evidence-index`, `valence-evidence-release`, `valence-object-evidence`, and Cairn validation run
- THEN they pass without weakening the existing bounded evidence contracts.

### Requirement: Phase 1 remains bounded and non-cutover
r[octet.valence_project_extraction.phase1.non_claims] The system MUST describe Phase 1 as a bounded in-repo crate seam and MUST NOT claim standalone repository cutover, package publication, correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

#### Scenario: No cutover or overclaim occurs
r[octet.valence_project_extraction.phase1.non_claims.explicit]
- GIVEN Phase 1 extracts a small deterministic evidence-index core seam
- WHEN the change is reviewed
- THEN it does not create a standalone repository, publish a package, move release tags, or broaden Valence evidence claims.
