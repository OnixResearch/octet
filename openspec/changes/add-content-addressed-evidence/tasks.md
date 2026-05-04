## Phase 1: Implementation

- [ ] I1 Add content addressed evidence standards documentation and policy classification
- [ ] I2 Add implementation hooks, command contracts, or templates required by the content addressed evidence design

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Finding hash fields appear on request" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-content-addressed-evidence/evidence/finding-hash-fields-appear-on-request.txt`
- [ ] V2 Verify scenario "Line moves do not define identity" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-content-addressed-evidence/evidence/line-moves-do-not-define-identity.txt`
- [ ] V3 Verify scenario "Evidence header is deterministic" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-content-addressed-evidence/evidence/evidence-header-is-deterministic.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review add-content-addressed-evidence proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/add-content-addressed-evidence/proposal.md, design.md, specs/consumer-integration/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/add-content-addressed-evidence/evidence/scope-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Finding hash fields appear on request` -> `V1` -> `openspec/changes/add-content-addressed-evidence/evidence/finding-hash-fields-appear-on-request.txt`
- `Scenario: Line moves do not define identity` -> `V2` -> `openspec/changes/add-content-addressed-evidence/evidence/line-moves-do-not-define-identity.txt`
- `Scenario: Evidence header is deterministic` -> `V3` -> `openspec/changes/add-content-addressed-evidence/evidence/evidence-header-is-deterministic.txt`
