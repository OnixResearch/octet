## Phase 1: Implementation

- [ ] I1 [serial] Update `openspec/templates/proposal.md`, `openspec/templates/spec.md`, and `openspec/templates/design.md` with omission-prevention prompts for the promoted proposal/spec/design omission classes.
- [ ] I2 [serial] Update `openspec/templates/tasks.md` with omission-prevention prompts for split implementation/proof tasks, one-row-per-scenario coverage, and exact `H<n>` replay fields.
- [ ] I3 [serial] Update `openspec/config.yaml` stage rules so proposal/spec/design/tasks prompts name the same omission-prevention obligations.
- [ ] I4 [serial] Add deterministic evidence helper `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`.

## Phase 2: Deterministic verification

- [ ] V1 [depends:I1] Verify scenario "Maintainer sees repeated proposal omission checks before drafting" with `python3 openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/proposal-template-omission-prompts.txt`.
- [ ] V2 [depends:I1] Verify scenario "Maintainer sees repeated delta-spec omission checks before review" with `python3 openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/spec-template-omission-prompts.txt`.
- [ ] V3 [depends:I1] Verify scenario "Maintainer sees repeated design omission checks before implementation" with `python3 openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/design-template-omission-prompts.txt`.
- [ ] V4 [depends:I3] Verify scenario "Config guidance includes promoted omission-prevention checks" with `python3 openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`.
- [ ] V5 [depends:I2] Verify scenario "Maintainer sees repeated task omission checks before gate review" with `python3 openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`.
- [ ] V6 [depends:I2] Verify scenario "Human checkpoint guidance blocks implicit oracle inputs" with `python3 openspec/changes/roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`.
- [ ] V7 [depends:I1] Verify active change validity with `openspec validate roi-01-openspec-omission-gate-rails --strict --json`; save output at `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/openspec-validate.txt`.

## Phase 3: Human/oracle checkpoints

- [ ] H1 [depends:V7] Human/oracle checkpoint: command=`openspec_gate stage=proposal change=roi-01-openspec-omission-gate-rails`; input=`proposal.md plus openspec-stage-authoring and openspec-task-governance delta specs`; asserts=`proposal-stage review accepts the omission-prevention scope and evidence plan`; evidence=`openspec/changes/roi-01-openspec-omission-gate-rails/evidence/proposal-gate.txt`
- [ ] H2 [depends:V7] Human/oracle checkpoint: command=`openspec_gate stage=design change=roi-01-openspec-omission-gate-rails`; input=`final design.md with scenario coverage and proof plan`; asserts=`design-stage review accepts the scenario coverage, boundary cases, and proof acceptance signals`; evidence=`openspec/changes/roi-01-openspec-omission-gate-rails/evidence/design-gate.txt`
- [ ] H3 [depends:V7] Human/oracle checkpoint: command=`openspec_gate stage=tasks change=roi-01-openspec-omission-gate-rails`; input=`tasks.md and delta specs`; asserts=`tasks-stage review accepts one task per scenario and exact H-task replay fields`; evidence=`openspec/changes/roi-01-openspec-omission-gate-rails/evidence/tasks-gate.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, I4, V1, V2, V3, V4, V5, V6, V7, H1, H2, H3
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: Maintainer sees repeated proposal omission checks before drafting` -> `V1` -> `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/proposal-template-omission-prompts.txt`
- `Scenario: Maintainer sees repeated delta-spec omission checks before review` -> `V2` -> `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/spec-template-omission-prompts.txt`
- `Scenario: Maintainer sees repeated design omission checks before implementation` -> `V3` -> `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/design-template-omission-prompts.txt`
- `Scenario: Config guidance includes promoted omission-prevention checks` -> `V4` -> `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`
- `Scenario: Maintainer sees repeated task omission checks before gate review` -> `V5` -> `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`
- `Scenario: Human checkpoint guidance blocks implicit oracle inputs` -> `V6` -> `openspec/changes/roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`
