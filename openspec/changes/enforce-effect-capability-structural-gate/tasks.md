## Phase 1: Implementation

- [ ] I1 Extend Nickel/Rust mechanism models with capability-api obligation fields
- [ ] I2 Export and load effect-capability obligations in generated policy artifacts
- [ ] I3 Update architecture checker to consume artifact-backed capability obligations
- [ ] I4 Add Tigerstyle-local compliant and noncompliant architecture fixtures
- [ ] I5 Update docs/check wiring for the effect capability structural gate

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Capability obligations are exported" with artifact export assertions; save output at `openspec/changes/enforce-effect-capability-structural-gate/evidence/policy-artifact-export.txt`
- [ ] V2 Verify scenario "Core ambient operation fails" with architecture noncompliant fixtures; save output at `openspec/changes/enforce-effect-capability-structural-gate/evidence/architecture-fixtures.txt`
- [ ] V3 Verify scenario "Injected core capability passes" with architecture compliant fixtures; save output at `openspec/changes/enforce-effect-capability-structural-gate/evidence/architecture-fixtures.txt`
- [ ] V4 Verify scenario "Shell constructor scope passes" with architecture compliant shell fixture; save output at `openspec/changes/enforce-effect-capability-structural-gate/evidence/architecture-fixtures.txt`
- [ ] V5 Verify focused Rust/Nix/OpenSpec checks; save output at `openspec/changes/enforce-effect-capability-structural-gate/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review design proof plan against tasks and delta spec`; input=`openspec/changes/enforce-effect-capability-structural-gate/{design.md,tasks.md,specs/standards-codification/spec.md}`; asserts=`no Aspen/downstream mutation proof is required for this Tigerstyle-local slice`; evidence=`openspec/changes/enforce-effect-capability-structural-gate/evidence/proof-plan-review.txt`

## Change Status

- Status: in-progress
- Open tasks: I1, I2, I3, I4, I5, V1, V2, V3, V4, V5, H1

## Verification Coverage

- `Scenario: Capability obligations are exported` -> `V1` -> `openspec/changes/enforce-effect-capability-structural-gate/evidence/policy-artifact-export.txt`
- `Scenario: Core ambient operation fails` -> `V2` -> `openspec/changes/enforce-effect-capability-structural-gate/evidence/architecture-fixtures.txt`
- `Scenario: Injected core capability passes` -> `V3` -> `openspec/changes/enforce-effect-capability-structural-gate/evidence/architecture-fixtures.txt`
- `Scenario: Shell constructor scope passes` -> `V4` -> `openspec/changes/enforce-effect-capability-structural-gate/evidence/architecture-fixtures.txt`
