## Phase 0: Stage gates

- [x] H1 [serial] Human/oracle checkpoint: command=`openspec_gate stage=proposal change=roi-02-dogfood-indexable-policy-lints`; input=`proposal.md plus consumer-integration delta spec`; asserts=`proposal-stage review accepts dogfood scope and canonical cargo-octet evidence plan`; evidence=`openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/proposal-gate.txt`; completed=2026-05-18T18:44:22Z
- [x] H2 [depends:H1] Human/oracle checkpoint: command=`openspec_gate stage=design change=roi-02-dogfood-indexable-policy-lints`; input=`design.md with dogfood helper and triage proof plan`; asserts=`design-stage review accepts command, cleanup boundary, and proof artifacts`; evidence=`openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/design-gate.txt`; completed=2026-05-18T18:44:22Z
- [x] H3 [depends:H2] Human/oracle checkpoint: command=`openspec_gate stage=tasks change=roi-02-dogfood-indexable-policy-lints`; input=`tasks.md and consumer-integration delta spec`; asserts=`tasks-stage review accepts split implementation, deterministic evidence, and triage checkpoint`; evidence=`openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/tasks-gate.txt`; completed=2026-05-18T18:44:22Z

## Phase 1: Implementation

- [ ] I1 [depends:H3] Add change-local dogfood helper `openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/run-indexable-dogfood.py`.
- [ ] I2 [depends:I1,V1] Apply narrow source/config cleanup for obvious dogfood findings, or document that no cleanup is needed.
- [ ] I3 [depends:I2] Write final triage notes for fixed or accepted findings.

## Phase 2: Deterministic verification

- [ ] V1 [depends:I1] Verify scenario "Dogfood run emits machine-readable finding inventory [r[indexable-policy-self-dogfood-inventory.scenario.machine-inventory]]" with `python3 openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/run-indexable-dogfood.py`; save output at `openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/dogfood-run.txt`, `dogfood-findings.jsonl`, and `dogfood-summary.txt`; assert the run uses canonical `cargo-octet`, enables exactly the nine archived indexable lints, disables non-target lints, and leaves no temporary dogfood config in the repo root.
- [ ] V2 [depends:I3] Verify scenario "Dogfood triage records fixed or accepted findings [r[indexable-policy-self-dogfood-inventory.scenario.triage-recorded]]" with `python3 openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/run-indexable-dogfood.py`; save final inventory plus `openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/dogfood-triage.txt`.
- [ ] V3 [depends:V2] Verify active change validity with `openspec validate roi-02-dogfood-indexable-policy-lints --strict --json`; save output at `openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/openspec-validate.txt`.

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, V1, V2, V3
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: Dogfood run emits machine-readable finding inventory [r[indexable-policy-self-dogfood-inventory.scenario.machine-inventory]]` -> `V1` -> `openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/dogfood-summary.txt`
- `Scenario: Dogfood triage records fixed or accepted findings [r[indexable-policy-self-dogfood-inventory.scenario.triage-recorded]]` -> `V2` -> `openspec/changes/roi-02-dogfood-indexable-policy-lints/evidence/dogfood-triage.txt`
