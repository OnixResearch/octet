## Phase 1: Implementation

- [ ] I1 Add the core DTOs and parsing/output seams for `content-identity-lint-cache`
- [ ] I2 Add positive and negative fixtures for `content-identity-lint-cache`
- [ ] I3 Update docs and examples for `content-identity-lint-cache`

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Unchanged item uses cached finding" with targeted fixture/helper; save output at `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-1.txt`
- [ ] V2 Verify scenario "Config change invalidates cache" with targeted fixture/helper; save output at `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-2.txt`
- [ ] V3 Verify scenario "Non-deterministic input bypasses cache" with targeted fixture/helper; save output at `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review the implemented content-identity-lint-cache UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-human-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Unchanged item uses cached finding` -> `V1` -> `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-1.txt`
- `Scenario: Config change invalidates cache` -> `V2` -> `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-2.txt`
- `Scenario: Non-deterministic input bypasses cache` -> `V3` -> `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-3.txt`
