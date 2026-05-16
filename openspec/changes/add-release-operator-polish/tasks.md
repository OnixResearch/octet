## Phase 1: Implementation

- [ ] I1 Update README/help docs to expose the release-critical operator surfaces
- [ ] I2 Add one release smoke transcript or docs-example fixture for the operator surfaces

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "New user chooses an install/run path" with README/help docs grep; save output at `openspec/changes/add-release-operator-polish/evidence/readme-run-paths.txt`
- [ ] V2 Verify scenario "Operator finds release-critical surfaces" with README/help docs grep; save output at `openspec/changes/add-release-operator-polish/evidence/operator-surfaces.txt`
- [ ] V3 Verify scenario "Release smoke transcript covers operator surfaces" with the docs-example/transcript gate; save output at `openspec/changes/add-release-operator-polish/evidence/release-smoke-transcript.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`review README/help and smoke transcript as a release operator`; input=`README.md, command help output, and release smoke transcript artifact`; asserts=`a release operator can find fingerprint, migrate, effect-summary, cache, and transcript surfaces without reading archived OpenSpec changes`; evidence=`openspec/changes/add-release-operator-polish/evidence/operator-review.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, V3, H1

## Verification Coverage

- `Scenario: New user chooses an install/run path` -> `V1` -> `openspec/changes/add-release-operator-polish/evidence/readme-run-paths.txt`
- `Scenario: Operator finds release-critical surfaces` -> `V2` -> `openspec/changes/add-release-operator-polish/evidence/operator-surfaces.txt`
- `Scenario: Release smoke transcript covers operator surfaces` -> `V3` -> `openspec/changes/add-release-operator-polish/evidence/release-smoke-transcript.txt`
