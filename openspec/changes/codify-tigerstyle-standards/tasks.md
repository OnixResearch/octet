## Phase 1: Implementation

- [x] I1 Migrate standards source material into `docs/standards/source/` with Tiger Style naming.
- [x] I2 Add `crates/tigerstyle-standards` pure checker core and CLI shell.
- [x] I3 Add `standards/policies/` Nickel registry, contracts, minimum coverage, and positive/negative registry fixtures.
- [x] I4 Add `standards/check-fixtures/` positive and negative fixtures for suppressions, architecture, and bounded resources.
- [x] I5 Wire Nix apps and checks for `tigerstyle-standards`, `tigerstyle-policy-export`, `tigerstyle-policy-validate`, `policy-registry`, and `standards-fixtures`.
- [x] I6 Update README, contributor guide, standards contract, and DevTools rail audit to name the Tiger Style standards seam.
- [ ] I7 Implement remaining architecture/public API/external-domain leakage checks after the foundation lands.
- [ ] I8 Implement remaining embedded/resilience/resource checks after fixture designs are narrowed.
- [ ] I9 Implement orchestration, idiom, test, telemetry, fitness, and safety-evidence reports in later waves.
- [ ] I10 Add final single-command standards verification entrypoint after all required rails are stable.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Legacy project names are absent from Tiger Style standards artifacts" with ripgrep; save output at `openspec/changes/codify-tigerstyle-standards/evidence/no-legacy-names.txt`.
- [x] V2 Verify scenarios "Valid Nickel policy exports for tools", "Invalid policy fails before checks run", and "Missing traceability is rejected" with `nix run .#tigerstyle-policy-validate`; save output at `openspec/changes/codify-tigerstyle-standards/evidence/policy-validate.txt`.
- [x] V3 Verify scenarios "Blocking hard rule fails deterministically", "Advisory rule reports without blocking", "Hard rule has fixture evidence", "Narrow suppression is accepted", and "Blanket suppression is rejected" with `cargo test -p tigerstyle-standards`; save output at `openspec/changes/codify-tigerstyle-standards/evidence/standards-tests.txt`.
- [x] V4 Verify scenarios "Fixture regression blocks CI" and "Local-first standards verification succeeds" with flake `policy-registry` and `standards-fixtures`; save output at `openspec/changes/codify-tigerstyle-standards/evidence/flake-standards-checks.txt`.
- [ ] V5 Verify the future scenario "Candidate rule with weak signal becomes advisory" after the remaining policy classification evidence is refreshed; save output at `openspec/changes/codify-tigerstyle-standards/evidence/advisory-classification.txt`.
- [ ] V6 Verify the final single-command standards entrypoint after I10 lands; save output at `openspec/changes/codify-tigerstyle-standards/evidence/final-standards-entrypoint.txt`.
- [x] V7 Verify OpenSpec artifact validity with `openspec validate codify-tigerstyle-standards --strict`; save output at `openspec/changes/codify-tigerstyle-standards/evidence/openspec-validate.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`AskClaude read-only staged-change review`; input=`staged Tiger Style standards migration`; asserts=`Dylint lint engine and standards policy checker remain separated by crate/command seams`; evidence=`openspec/changes/codify-tigerstyle-standards/evidence/secondary-review.txt`.

## Change Status
- Status: in-progress
- Open tasks: I7, I8, I9, I10, V5, V6
## Verification Coverage

- `Scenario: Codifiable source becomes policy` -> `V2` -> `openspec/changes/codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Human-only guidance stays explicit` -> `V5` -> `openspec/changes/codify-tigerstyle-standards/evidence/advisory-classification.txt`
- `Scenario: Missing traceability is rejected` -> `V2` -> `openspec/changes/codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Valid Nickel policy exports for tools` -> `V2` -> `openspec/changes/codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Invalid policy fails before checks run` -> `V2` -> `openspec/changes/codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Blocking hard rule fails deterministically` -> `V3` -> `openspec/changes/codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Advisory rule reports without blocking` -> `V3` -> `openspec/changes/codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Candidate rule with weak signal becomes advisory` -> `V5` -> `openspec/changes/codify-tigerstyle-standards/evidence/advisory-classification.txt`
- `Scenario: Hard rule has fixture evidence` -> `V3` -> `openspec/changes/codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Narrow suppression is accepted` -> `V3` -> `openspec/changes/codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Blanket suppression is rejected` -> `V3` -> `openspec/changes/codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Local-first standards verification succeeds` -> `V4` -> `openspec/changes/codify-tigerstyle-standards/evidence/flake-standards-checks.txt`
- `Scenario: Fixture regression blocks CI` -> `V4` -> `openspec/changes/codify-tigerstyle-standards/evidence/flake-standards-checks.txt`
- `Scenario: Legacy project names are absent from Tiger Style standards artifacts` -> `V1` -> `openspec/changes/codify-tigerstyle-standards/evidence/no-legacy-names.txt`
