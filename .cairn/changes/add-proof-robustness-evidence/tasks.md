# Tasks

## Phase 1: Contract and identity

- [ ] [serial] Record the pinned Dafny design-reference identity, reviewed license boundary, selected source mechanisms, and clean-room adaptation rule. r[octet.proof_robustness.boundary]
- [ ] [serial] Define the typed Nickel robustness profile and deterministic runtime export. r[octet.proof_robustness.profile]
- [ ] [serial] Define Rust-owned run, audit, proof-use, classification, and receipt DTOs with generated Nickel contracts. r[octet.proof_robustness.receipt]

## Phase 2: Core and shell

- [ ] [serial] Implement pure profile, run-set, metric, audit, dependency-coverage, identity, and classification validation. r[octet.proof_robustness.classification] r[octet.proof_robustness.assumption_audit] r[octet.proof_robustness.proof_use]
- [ ] [serial] Extend the existing proof planner with ordered seed-matrix plans and bounded provider metric capture. r[octet.proof_robustness.execution]
- [ ] [serial] Add the Verus adapter for explicit solver seeds, profile metrics, no-cheating mode, and axiom-usage facts. r[octet.proof_robustness.execution] r[octet.proof_robustness.assumption_audit] r[octet.proof_robustness.proof_use]
- [ ] [serial] Extend `cargo octet prove` with an opt-in robustness profile while preserving ordinary proof and starter behavior. r[octet.proof_robustness.profile] r[octet.proof_robustness.boundary]
- [ ] [serial] Extend artifact validation and existing Valence export with linked robustness receipts and unchanged evidence roles. r[octet.proof_robustness.receipt]

## Phase 3: Positive and negative evidence

- [ ] [parallel] Add positive fixtures for stable proof outcomes, admitted resource variation, declared boundaries, axiom-use-only coverage, and byte-stable receipts. r[octet.proof_robustness.verification]
- [ ] [parallel] Add negative fixtures for changed outcomes, missing seeds, duplicate seeds, metric drift, stale child receipts, changed assumptions, unsupported dependency promotion, timeout, and overclaims. r[octet.proof_robustness.verification]
- [ ] [parallel] Add a provider fixture that reports unsupported proof-use data without fabricating an empty dependency set. r[octet.proof_robustness.proof_use] r[octet.proof_robustness.verification]

## Phase 4: Documentation and closeout

- [ ] [serial] Document profile selection, classifications, assumption visibility, proof-use coverage, artifact verification, and non-claims. r[octet.proof_robustness.boundary]
- [ ] [serial] Run focused tests, contract freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage. r[octet.proof_robustness.verification]
- [ ] [serial] Run proposal, design, and tasks gates before sync and archive. r[octet.proof_robustness.profile]
