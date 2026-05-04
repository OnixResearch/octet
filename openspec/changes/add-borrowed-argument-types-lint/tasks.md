## Phase 1: OpenSpec foundation

- [x] I1 Create the focused child OpenSpec for the borrowed argument types lint candidate
- [x] V1 Verify the child OpenSpec validates strictly; evidence=`openspec validate add-borrowed-argument-types-lint --strict`
- [x] V2 Verify no active parent change remains and the new child is the only active change; evidence=`python helper/inventory active changes`

## Phase 2: Implementation

- [ ] I2 Implement the `borrowed_argument_types` lint in `src/structure/` and register it in the lint inventory
- [ ] I3 Add diagnostic metadata, catalog/docs entries, and configuration naming for the new lint
- [ ] I4 Add compliant, noncompliant, and false-positive-boundary UI fixtures for `&String`, `&Vec<T>`, `&PathBuf`, trait-dictated signatures, extern boundaries, and already-borrowed forms

## Phase 3: Verification

- [ ] V3 Run focused UI tests for the new fixtures; evidence=`openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
- [ ] V4 Run package-level checks required for lint registration changes; evidence=`openspec/changes/add-borrowed-argument-types-lint/evidence/package-checks.txt`
- [ ] V5 Run policy/export checks proving the lint remains advisory or allow-by-default until fixture evidence supports promotion; evidence=`openspec/changes/add-borrowed-argument-types-lint/evidence/policy-advisory-gate.txt`

## Change Status
- Status: active
- Open tasks: I2, I3, I4, V3, V4, V5

## Verification Coverage

- `Scenario: String reference parameter` -> `I4,V3` -> `openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
- `Scenario: Vec reference parameter` -> `I4,V3` -> `openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
- `Scenario: PathBuf reference parameter` -> `I4,V3` -> `openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
- `Scenario: Trait dictated signature is skipped` -> `I4,V3` -> `openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
- `Scenario: Extern ABI signature is skipped` -> `I4,V3` -> `openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
- `Scenario: Already borrowed forms are accepted` -> `I4,V3` -> `openspec/changes/add-borrowed-argument-types-lint/evidence/ui-borrowed-argument-types.txt`
