## Phase 1: Implementation

- [ ] I1 Add the core DTOs and parsing/output seams for `executable-doc-transcripts`
- [ ] I2 Add positive and negative fixtures for `executable-doc-transcripts`
- [ ] I3 Update docs and examples for `executable-doc-transcripts`

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Transcript executes command blocks" with targeted fixture/helper; save output at `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-1.txt`
- [ ] V2 Verify scenario "Expected output assertion fails closed" with targeted fixture/helper; save output at `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-2.txt`
- [ ] V3 Verify scenario "Transcript artifact is content addressed" with targeted fixture/helper; save output at `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review the implemented executable-doc-transcripts UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-human-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, V1, V2, V3, H1

## Verification Coverage

- `Scenario: Transcript executes command blocks` -> `V1` -> `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-1.txt`
- `Scenario: Expected output assertion fails closed` -> `V2` -> `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-2.txt`
- `Scenario: Transcript artifact is content addressed` -> `V3` -> `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-3.txt`
