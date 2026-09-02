# Tasks

## Phase 1: Contract

- [ ] [serial] Record the pinned Dafny workspace-state reference, reviewed license boundary, and clean-room adaptation rule. r[octet.proof_workspace_state.dafny_boundary]
- [ ] [serial] Define Rust-owned event, generation, node, transition, diagnostic, and snapshot DTOs with a generated Nickel contract. r[octet.proof_workspace_state.contract]
- [ ] [serial] Define typed policy for node, depth, event, text, and artifact limits. r[octet.proof_workspace_state.contract]

## Phase 2: Core and shell

- [ ] [serial] Implement the pure ordered event reducer and closed state-transition table. r[octet.proof_workspace_state.transitions]
- [ ] [serial] Implement source, proof, policy, verifier, and receipt generation matching with stale-result invalidation. r[octet.proof_workspace_state.freshness]
- [ ] [serial] Add bounded snapshot construction and deterministic diagnostics for malformed trees and event order. r[octet.proof_workspace_state.snapshot]
- [ ] [serial] Extend `cargo octet prove` with opt-in workspace-state events and atomic snapshot output. r[octet.proof_workspace_state.shell]
- [ ] [serial] Link only current matching terminal receipts to existing proof goal-state and artifact views. r[octet.proof_workspace_state.freshness] r[octet.proof_workspace_state.boundary]

## Phase 3: Positive and negative evidence

- [ ] [parallel] Add positive fixtures for pass, fail, blocked, cancelled, unsupported, generation replacement, and deterministic replay. r[octet.proof_workspace_state.verification]
- [ ] [parallel] Add negative fixtures for sequence gaps, regressions, duplicate terminal events, missing parents, cycles, unknown states, stale receipt promotion, and exceeded bounds. r[octet.proof_workspace_state.verification]
- [ ] [parallel] Add a consumer fixture that renders state without mutating the tree or assigning proof authority. r[octet.proof_workspace_state.boundary]

## Phase 4: Documentation and closeout

- [ ] [serial] Document the schema, transitions, freshness rules, event replay, artifact output, accessibility labels, and non-claims. r[octet.proof_workspace_state.boundary]
- [ ] [serial] Run focused tests, contract freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage. r[octet.proof_workspace_state.verification]
- [ ] [serial] Run proposal, design, and tasks gates before sync and archive. r[octet.proof_workspace_state.contract]
