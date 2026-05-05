## Phase 1: Implementation

- [ ] I1 Extend Nickel and Rust policy models with generated type-state mechanism obligation fields
- [ ] I2 Export type-state obligations through generated policy artifacts and checker config
- [ ] I3 Implement artifact-backed type-state structural diagnostics in `tigerstyle-standards`
- [ ] I4 Add compliant and noncompliant type-state protocol fixtures
- [ ] I5 Update docs describing type-state structural mechanism artifacts

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Type-state obligations export deterministically" with artifact export assertions; save output at `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/policy-artifact-export.txt`
- [ ] V2 Verify scenario "Compliant nominal marker-state API passes" with standards fixture check; save output at `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- [ ] V3 Verify scenario "Raw protocol bypass fails structurally" with standards fixture check; save output at `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- [ ] V4 Verify scenario "Focused Tigerstyle checks pass" with Rust/Nix/OpenSpec commands; save output at `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review design proof plan before archive`; input=`openspec/changes/enforce-type-state-protocol-structural-gate/design.md and evidence transcripts`; asserts=`the change claims structural API enforcement only, not verified protocol correctness`; evidence=`openspec/changes/enforce-type-state-protocol-structural-gate/evidence/proof-plan-review.txt`

## Change Status

- Status: in-progress
- Open tasks: I1, I2, I3, I4, I5, V1, V2, V3, V4, H1

## Verification Coverage

- `Scenario: Type-state obligations export deterministically` -> `V1` -> `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/policy-artifact-export.txt`
- `Scenario: Compliant nominal marker-state API passes` -> `V2` -> `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- `Scenario: Raw protocol bypass fails structurally` -> `V3` -> `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- `Scenario: Focused Tigerstyle checks pass` -> `V4` -> `openspec/changes/enforce-type-state-protocol-structural-gate/evidence/focused-checks.txt`
