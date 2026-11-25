# Tasks: Location-independent proof-deck locators

## Phase 1: Resolution surface

- [x] [serial] Add `--artifact-base PATH` to `requirements check` and `requirements verify`: the CLI arg structs, the parser, the usage text, and the unsupported-argument message. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Thread `artifact_base` through `evaluate_contract` and the clause `Context` into `formal::evaluate`, and add one `resolve_artifact_locator` helper in `cargo-octet/src/requirements.rs` that keeps `resolve_path` behavior when no base is given. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Apply the containment rule to relative `artifact_root` and `deck` locators: refuse empty, root, prefix, and `..` components, and refuse an absolute locator inside the base whose stripped remainder escapes it. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Add tests for two identical deck trees in different directories producing byte-identical decisions, escaping relative and absolute-inside-base locators being refused, absolute locators keeping today's behavior, and verify recomputing the same identity and reporting a mismatch for different proof-card bytes. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Extend the CLI parser test to assert `--artifact-base` on both requirements actions. r[octet.cairn_convergence_pilot.locators]

## Phase 2: Fixture and documentation cutover

- [x] [serial] Convert `standards/requirements/fixtures/cairn-pilot/evidence.json` to `artifact_root: "."` and `deck: "artifacts/proof-cards.json"`, and rewrite its non-claim about absolute paths. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Regenerate `standards/requirements/fixtures/cairn-pilot/decision.json` with `requirements check --artifact-base` over the preserved realized output and require the new decision bytes. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Update the `octet.cairn-pilot-requirement-contract` registry validation command, the pilot README, `docs/requirements-contracts.md`, CLI help, and the CHANGELOG entry. r[octet.cairn_convergence_pilot.locators]

## Phase 3: Evidence

- [x] [serial] Run the focused `cargo-octet` requirements tests, `cargo fmt --check`, and `cargo clippy -p cargo-octet` inside `nix develop`, and save the transcripts under `evidence/`. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Run `requirements check` and `requirements verify` against the pilot fixture with `--artifact-base`, `cmp` the decision with the committed fixture, and save the transcript. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Copy the realized store's `artifacts/` proof-card tree to a second directory, evaluate both with `--artifact-base`, and save the byte-identity and escaping-locator transcripts. r[octet.cairn_convergence_pilot.locators]
- [x] [serial] Run `cairn validate --strict`, the proposal/design/tasks gates, and `cargo test -p octet-standards` for the touched registry surface, and save the receipts and transcripts. r[octet.cairn_convergence_pilot.locators]

## Verification Coverage

- `Scenario: Identical proof cards at different locations produce one decision` -> `evidence/two-location-identity.txt`
- `Scenario: Relative locator escaping the artifact base is refused` -> `evidence/escaping-locator.txt`
- `Scenario: Absolute locators keep their pre-existing resolution` -> `evidence/requirements-check.txt`
- `Scenario: Verify recomputes the recorded decision` -> `evidence/requirements-verify.txt`
- `Scenario: Different proof-card bytes are reported as a mismatch` -> `evidence/verify-mismatch.txt`
