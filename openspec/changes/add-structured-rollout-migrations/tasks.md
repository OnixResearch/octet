## Phase 1: Implementation

- [ ] I1 Add the core DTOs and parsing/output seams for `structured-rollout-migrations`
- [ ] I2 Add positive and negative fixtures for `structured-rollout-migrations`
- [ ] I3 Update docs and examples for `structured-rollout-migrations`

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Migration start writes deterministic manifest" with targeted fixture/helper; save output at `openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-scenario-1.txt`
- [ ] V2 Verify scenario "Migration status reports remaining work" with targeted fixture/helper; save output at `openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-scenario-2.txt`
- [ ] V3 Verify scenario "Safe apply does not modify manual-review findings" with targeted fixture/helper; save output at `openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review the implemented structured-rollout-migrations UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-human-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Migration start writes deterministic manifest` -> `V1` -> `openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-scenario-1.txt`
- `Scenario: Migration status reports remaining work` -> `V2` -> `openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-scenario-2.txt`
- `Scenario: Safe apply does not modify manual-review findings` -> `V3` -> `openspec/changes/add-structured-rollout-migrations/evidence/add-structured-rollout-migrations-scenario-3.txt`
