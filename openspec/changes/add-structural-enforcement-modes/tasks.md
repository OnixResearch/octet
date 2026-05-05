Spec-foundation note: this package defines future implementation and verification work only; no implementation task is complete yet.

## Phase 1: Implementation

- [ ] I1 Extend the policy registry schema/export with current and target enforcement modes plus mechanism metadata.
- [ ] I2 Implement policy validation for unknown modes, missing escalation criteria, missing mechanisms, and invalid fallback declarations.
- [ ] I3 Implement the first structural core/shell checker slice for no-std core declarations, forbidden core effects, and shell-to-core dependency direction.
- [ ] I4 Add capability-constructor validation for approved shell-owned authority wrappers and injected core values.
- [ ] I5 Add type-state and bounded-primitive validation fixtures and checker hooks for configured structural policies.
- [ ] I6 Add verified-mode metadata and proof-rail validation for backend, command, source/proof mapping, pass signal, and trust-boundary accounting.
- [ ] I7 Add reference seed manifest support for optional imports from sibling repositories.
- [ ] I8 Add minimal UCAN-style, verified-logic-style, and Vericoder-style seed fixtures or vendored examples only after manifest validation exists.
- [ ] I9 Update docs, templates, and machine-output metadata to distinguish lint, gate, structural, verified, and human/oracle findings.

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Lint-only policy remains explicit" with policy-schema fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- [ ] V2 Verify scenario "Target mode stronger than current mode records escalation" with policy-schema fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- [ ] V3 Verify scenario "No-std core boundary is enforced" with core/shell structural fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- [ ] V4 Verify scenario "Shell-to-core dependency direction is enforced" with crate-graph fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- [ ] V5 Verify scenario "Capability constructors are shell-owned" with positive and negative capability fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/capability-structural-fixtures.txt`
- [ ] V6 Verify scenario "Type-state protocol surfaces reject raw operations" with positive and negative type-state fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- [ ] V7 Verify scenario "Bounded primitive requirement rejects unbounded replacement" with positive and negative bounded primitive fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- [ ] V8 Verify scenario "Verified policy requires proof command" with verified-mode manifest fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- [ ] V9 Verify scenario "Proof-relevant code cannot be imported as trusted by default" with seed manifest negative fixture; save output at `openspec/changes/add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- [ ] V10 Verify scenario "Vericoder seed is accepted as architecture or trajectory fixture" with seed manifest fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- [ ] V11 Verify scenario "Verified-logic seed requires local proof boundary" with proof-relevant seed manifest fixtures; save output at `openspec/changes/add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- [ ] V12 Verify scenario "UCAN seed demonstrates core-shell vendoring shape" with UCAN-style seed fixture; save output at `openspec/changes/add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- [ ] V13 Verify scenario "Machine output classifies enforcement failure type" with machine-output fixture parsing; save output at `openspec/changes/add-structural-enforcement-modes/evidence/enforcement-output.txt`
- [ ] V14 Verify scenario "Nix or CI gate fails on missing structural evidence" with local-first Nix/check fixture; save output at `openspec/changes/add-structural-enforcement-modes/evidence/enforcement-output.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`manual review of seed manifest and vendored paths before accepting any sibling import`; input=`source commits and manifests for ../vericoder-mvp/, ../verified-logic/, and ../ucan/`; asserts=`license/provenance and role classification are sufficient for every vendored seed`; evidence=`openspec/changes/add-structural-enforcement-modes/evidence/seed-provenance-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, I4, I5, I6, I7, I8, I9, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, H1
## Verification Coverage

- `Scenario: Lint-only policy remains explicit` -> `V1` -> `openspec/changes/add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- `Scenario: Target mode stronger than current mode records escalation` -> `V2` -> `openspec/changes/add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- `Scenario: No-std core boundary is enforced` -> `V3` -> `openspec/changes/add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- `Scenario: Shell-to-core dependency direction is enforced` -> `V4` -> `openspec/changes/add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- `Scenario: Capability constructors are shell-owned` -> `V5` -> `openspec/changes/add-structural-enforcement-modes/evidence/capability-structural-fixtures.txt`
- `Scenario: Type-state protocol surfaces reject raw operations` -> `V6` -> `openspec/changes/add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- `Scenario: Bounded primitive requirement rejects unbounded replacement` -> `V7` -> `openspec/changes/add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- `Scenario: Verified policy requires proof command` -> `V8` -> `openspec/changes/add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- `Scenario: Proof-relevant code cannot be imported as trusted by default` -> `V9` -> `openspec/changes/add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- `Scenario: Vericoder seed is accepted as architecture or trajectory fixture` -> `V10` -> `openspec/changes/add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- `Scenario: Verified-logic seed requires local proof boundary` -> `V11` -> `openspec/changes/add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- `Scenario: UCAN seed demonstrates core-shell vendoring shape` -> `V12` -> `openspec/changes/add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- `Scenario: Machine output classifies enforcement failure type` -> `V13` -> `openspec/changes/add-structural-enforcement-modes/evidence/enforcement-output.txt`
- `Scenario: Nix or CI gate fails on missing structural evidence` -> `V14` -> `openspec/changes/add-structural-enforcement-modes/evidence/enforcement-output.txt`
