## Phase 1: Machine-applicable fixes

- [ ] `compound_assertion`: emit a `multipart_suggestion` only for validated top-level `assert!(a && b)` cases
- [ ] Add `.fixed` proof for every `compound_assertion` pattern marked `MachineApplicable`
- [ ] Add an indentation-sensitive `compound_assertion` fixture proving the multipart rewrite preserves source indentation in snapshots or `.fixed` output
- [ ] Add a guardrail fixture proving top-level `assert!(a || b)` does not overclaim `MachineApplicable`

## Phase 2: Review-required (`MaybeIncorrect`) suggestions

- [ ] `bool_naming`: declaration-scoped rename suggestion with review-required applicability unless use-site propagation lands in the same change
- [ ] `acronym_style`: declaration-scoped rename suggestion with review-required applicability unless use-site propagation lands in the same change
- [ ] `unchecked_division`: suggest `assert!(b != 0);` insertion
- [ ] `raw_arithmetic_overflow`: suggest checked or saturating arithmetic replacements
- [ ] `unchecked_narrowing`: suggest `u32::try_from(value)?` replacement
- [ ] `platform_dependent_cast`: suggest `usize::try_from(value)?` replacement
- [ ] Add or update UI fixtures so `.stderr` snapshots prove the suggestion text and review-required applicability for every Phase 2 lint

## Phase 3: Placeholder (`HasPlaceholders`) suggestions

- [ ] `numeric_units`: suggest a unit-bearing rename with explicit placeholder choices
- [ ] `unjustified_allow`: suggest a stub justification comment that still requires maintainer-written rationale
- [ ] Add or update UI fixtures so `.stderr` snapshots prove placeholder suggestion text for every Phase 3 lint

## Phase 4: Validation artifacts

- [ ] Save ignored UI proof for the full rollout fixture set under `openspec/changes/autofix-suggestions/evidence/cargo-test-ui-ignored.txt`, with each exercised autofix fixture name printed and the covered snapshot set made explicit in the saved artifact
- [ ] Save the supported nix-devshell `cargo fix` proof transcript under `openspec/changes/autofix-suggestions/evidence/cargo-fix-proof.txt`, including the exact proof command and machine-applicable target exercised
- [ ] Update all `.stderr` expectations to include suggestion text and applicability notes
- [ ] Verify every `.fixed` proof file compiles without warnings
- [ ] Add a guardrail check proving the diagnostics-only and unchanged out-of-rollout lints named in `specs/autofix/spec.md` stay outside autofix artifacts and implementation scope for this change
