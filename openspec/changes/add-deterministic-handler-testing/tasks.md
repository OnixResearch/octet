## Phase 1: Implementation

- [ ] I1 Add deterministic handler testing standards documentation and policy classification
- [ ] I2 Add implementation hooks, command contracts, or templates required by the deterministic handler testing design

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Handler test is ambient-free" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-deterministic-handler-testing/evidence/handler-test-is-ambient-free.txt`
- [ ] V2 Verify scenario "Recording handler proves requested effects" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-deterministic-handler-testing/evidence/recording-handler-proves-requested-effects.txt`
- [ ] V3 Verify scenario "Failure handler proves cleanup" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-deterministic-handler-testing/evidence/failure-handler-proves-cleanup.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review add-deterministic-handler-testing proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/add-deterministic-handler-testing/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/add-deterministic-handler-testing/evidence/scope-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Handler test is ambient-free` -> `V1` -> `openspec/changes/add-deterministic-handler-testing/evidence/handler-test-is-ambient-free.txt`
- `Scenario: Recording handler proves requested effects` -> `V2` -> `openspec/changes/add-deterministic-handler-testing/evidence/recording-handler-proves-requested-effects.txt`
- `Scenario: Failure handler proves cleanup` -> `V3` -> `openspec/changes/add-deterministic-handler-testing/evidence/failure-handler-proves-cleanup.txt`
