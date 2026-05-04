## Phase 1: Implementation

- [ ] I1 Add effect capability boundaries standards documentation and policy classification
- [ ] I2 Add implementation hooks, command contracts, or templates required by the effect capability boundaries design

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Core effects are explicit" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-effect-capability-boundaries/evidence/core-effects-are-explicit.txt`
- [ ] V2 Verify scenario "Shell interprets capabilities" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-effect-capability-boundaries/evidence/shell-interprets-capabilities.txt`
- [ ] V3 Verify scenario "Lint candidates stay staged" with deterministic docs/policy/fixture checks; save output at `openspec/changes/add-effect-capability-boundaries/evidence/lint-candidates-stay-staged.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review add-effect-capability-boundaries proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/add-effect-capability-boundaries/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/add-effect-capability-boundaries/evidence/scope-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Core effects are explicit` -> `V1` -> `openspec/changes/add-effect-capability-boundaries/evidence/core-effects-are-explicit.txt`
- `Scenario: Shell interprets capabilities` -> `V2` -> `openspec/changes/add-effect-capability-boundaries/evidence/shell-interprets-capabilities.txt`
- `Scenario: Lint candidates stay staged` -> `V3` -> `openspec/changes/add-effect-capability-boundaries/evidence/lint-candidates-stay-staged.txt`
