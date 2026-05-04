## Phase 1: Implementation

- [ ] I1 Add semantic refactor evidence standards documentation and policy classification
- [ ] I2 Add implementation hooks, command contracts, or templates required by the semantic refactor evidence design

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Move record preserves identity" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-semantic-refactor-evidence/evidence/move-record-preserves-identity.txt`
- [ ] V2 Verify scenario "Rename record states rationale" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-semantic-refactor-evidence/evidence/rename-record-states-rationale.txt`
- [ ] V3 Verify scenario "Machine output is stable" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-semantic-refactor-evidence/evidence/machine-output-is-stable.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review add-semantic-refactor-evidence proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/add-semantic-refactor-evidence/proposal.md, design.md, specs/consumer-integration/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/add-semantic-refactor-evidence/evidence/scope-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Move record preserves identity` -> `V1` -> `openspec/changes/add-semantic-refactor-evidence/evidence/move-record-preserves-identity.txt`
- `Scenario: Rename record states rationale` -> `V2` -> `openspec/changes/add-semantic-refactor-evidence/evidence/rename-record-states-rationale.txt`
- `Scenario: Machine output is stable` -> `V3` -> `openspec/changes/add-semantic-refactor-evidence/evidence/machine-output-is-stable.txt`
