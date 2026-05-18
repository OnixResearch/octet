## Phase 1: Implementation

- [x] I1 Define the Octet naming map in README/docs and distinguish canonical identity from Tiger Style philosophy and legacy compatibility. ✅ 3m20s (started: 2026-05-18T01:49:52Z → completed: 2026-05-18T01:53:12Z)
- [x] I2 Rename Cargo package/binary/runner surfaces to `octet` / `cargo-octet` while retaining explicit compatibility aliases where chosen. ✅ 6m50s (started: 2026-05-18T01:54:05Z → completed: 2026-05-18T02:00:55Z)
- [x] I3 Update Nix packages/apps/check helpers, flake library surfaces, and generated command wrappers to prefer Octet names. ✅ 12m (started: 2026-05-18T02:07:25Z → completed: 2026-05-18T02:19:25Z; evidence: `openspec/changes/rename-slotcar-to-octet/evidence/metadata-build.txt`)
- [x] I4 Update consumer config parsing/templates/docs for `[octet]`, `[octet.lint_levels]`, and `[workspace.metadata.octet]` with Octet-over-legacy precedence. ✅ 10m (started: 2026-05-18T02:21:32Z → completed: 2026-05-18T02:31:32Z; evidence: `openspec/changes/rename-slotcar-to-octet/evidence/config-surface.txt`)
- [x] I5 Update structured outputs, SARIF driver metadata, sidecar artifacts, migration manifests, transcript reports, and fingerprint/review records to use Octet canonical identity. ✅ 6m38s (started: 2026-05-18T02:38:42Z → completed: 2026-05-18T02:45:20Z; evidence: `openspec/changes/rename-slotcar-to-octet/evidence/structured-output.txt`)
- [x] I6 Update standards registry/policy artifact names and regenerate any Nickel-derived artifacts affected by Octet identity. ✅ 19m39s (started: 2026-05-18T03:17:44Z → completed: 2026-05-18T03:37:23Z; evidence: `openspec/changes/rename-slotcar-to-octet/evidence/policy-artifacts.txt`)
- [ ] I7 Update tests, UI snapshots, consumer fixtures, docs examples, and changelog entries for the Octet identity.
- [ ] I8 Add deterministic remaining-name classification support for non-archived `slotcar`, `Slotcar`, `tigerstyle`, and `Tiger Style` occurrences.

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Canonical install and usage docs say Octet" with the remaining-name classification helper; save output at `openspec/changes/rename-slotcar-to-octet/evidence/name-classification.txt`
- [ ] V2 Verify scenario "Chemistry metaphor explains stable-shell interfaces" with a focused docs grep/helper; save output at `openspec/changes/rename-slotcar-to-octet/evidence/docs-metaphor.txt`
- [ ] V3 Verify scenario "Legacy names remain distinguishable" with the remaining-name classification helper; save output at `openspec/changes/rename-slotcar-to-octet/evidence/name-classification.txt`
- [ ] V4 Verify scenario "Cargo subcommand uses Octet" with focused cargo runner help/integration checks; save output at `openspec/changes/rename-slotcar-to-octet/evidence/cli-surface.txt`
- [ ] V5 Verify scenario "Structured outputs name Octet" with focused human/JSON/SARIF output checks; save output at `openspec/changes/rename-slotcar-to-octet/evidence/structured-output.txt`
- [ ] V6 Verify scenario "Compatibility aliases are explicit" with compatibility invocation tests or a documented no-alias decision; save output at `openspec/changes/rename-slotcar-to-octet/evidence/compatibility-aliases.txt`
- [ ] V7 Verify scenario "Octet config section controls lint behavior" with a focused config fixture/integration test; save output at `openspec/changes/rename-slotcar-to-octet/evidence/config-surface.txt`
- [ ] V8 Verify scenario "Octet workspace metadata controls runner policy" with a focused consumer integration test; save output at `openspec/changes/rename-slotcar-to-octet/evidence/config-surface.txt`
- [ ] V9 Verify scenario "Octet config wins over legacy config" with a focused precedence test; save output at `openspec/changes/rename-slotcar-to-octet/evidence/config-precedence.txt`
- [ ] V10 Verify the rename package/build metadata with `cargo metadata --no-deps`, `git diff --check`, and the smallest maintained Nix/Cargo checks that cover renamed surfaces; save output at `openspec/changes/rename-slotcar-to-octet/evidence/metadata-build.txt`
- [x] V11 Verify any Nickel/policy artifact rename effects with policy export/validation checks when `.ncl` files change. ✅ 20m02s (started: 2026-05-18T03:17:44Z → completed: 2026-05-18T03:37:46Z; evidence: `openspec/changes/rename-slotcar-to-octet/evidence/policy-artifacts.txt`)

## Change Status
- Status: in-progress
- Open tasks: I7, I8, V1, V2, V3, V4, V5, V6, V7, V8, V9, V10

## Verification Coverage

- `Scenario: Canonical install and usage docs say Octet` -> `V1` -> `openspec/changes/rename-slotcar-to-octet/evidence/name-classification.txt`
- `Scenario: Chemistry metaphor explains stable-shell interfaces` -> `V2` -> `openspec/changes/rename-slotcar-to-octet/evidence/docs-metaphor.txt`
- `Scenario: Legacy names remain distinguishable` -> `V3` -> `openspec/changes/rename-slotcar-to-octet/evidence/name-classification.txt`
- `Scenario: Cargo subcommand uses Octet` -> `V4` -> `openspec/changes/rename-slotcar-to-octet/evidence/cli-surface.txt`
- `Scenario: Structured outputs name Octet` -> `V5` -> `openspec/changes/rename-slotcar-to-octet/evidence/structured-output.txt`
- `Scenario: Compatibility aliases are explicit` -> `V6` -> `openspec/changes/rename-slotcar-to-octet/evidence/compatibility-aliases.txt`
- `Scenario: Octet config section controls lint behavior` -> `V7` -> `openspec/changes/rename-slotcar-to-octet/evidence/config-surface.txt`
- `Scenario: Octet workspace metadata controls runner policy` -> `V8` -> `openspec/changes/rename-slotcar-to-octet/evidence/config-surface.txt`
- `Scenario: Octet config wins over legacy config` -> `V9` -> `openspec/changes/rename-slotcar-to-octet/evidence/config-precedence.txt`
