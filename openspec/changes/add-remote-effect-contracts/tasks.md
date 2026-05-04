## Phase 1: Implementation

- [ ] I1 Add remote effect contracts standards documentation and policy classification
- [ ] I2 Add implementation hooks, command contracts, or templates required by the remote effect contracts design

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Remote task carries bounds" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-remote-effect-contracts/evidence/remote-task-carries-bounds.txt`
- [ ] V2 Verify scenario "Receipt semantics are explicit" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-remote-effect-contracts/evidence/receipt-semantics-are-explicit.txt`
- [ ] V3 Verify scenario "Weak static signal remains human reviewed" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-remote-effect-contracts/evidence/weak-static-signal-remains-human-reviewed.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review add-remote-effect-contracts proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/add-remote-effect-contracts/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/add-remote-effect-contracts/evidence/scope-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Remote task carries bounds` -> `V1` -> `openspec/changes/add-remote-effect-contracts/evidence/remote-task-carries-bounds.txt`
- `Scenario: Receipt semantics are explicit` -> `V2` -> `openspec/changes/add-remote-effect-contracts/evidence/receipt-semantics-are-explicit.txt`
- `Scenario: Weak static signal remains human reviewed` -> `V3` -> `openspec/changes/add-remote-effect-contracts/evidence/weak-static-signal-remains-human-reviewed.txt`
