## Phase 1: Profiles and contracts

- [ ] [serial] Add typed Nickel cargo-mutants profiles with exact scope, provider identity, fixed command shape, budgets, limits, retention, and non-claims r[octet.mutation_testing.profile]
- [ ] [serial] Define Rust-owned baseline, inventory, outcome, and receipt DTOs with generated Nickel contracts r[octet.mutation_testing.receipt]
- [ ] [serial] Register profile, native output, normalized projection, logs, diffs, receipt, and provenance artifact ownership r[octet.mutation_testing.receipt]

## Phase 2: Core and shell

- [ ] [serial] Implement pure profile, baseline, inventory, outcome-partition, identity, limit, and gate-policy validation r[octet.mutation_testing.receipt]
- [ ] [serial] Add fixed profile-derived cargo-mutants execution with capability-rooted outputs and bounded process handling r[octet.mutation_testing.profile]
- [ ] [serial] Normalize only the pinned provider's declared machine-readable artifacts and reject human-log scraping r[octet.mutation_testing.native_artifacts]
- [ ] [serial] Require and bind a matching successful unmutated baseline before passing mutation classification r[octet.mutation_testing.baseline]
- [ ] [serial] Preserve compatible legacy imports as bounded transcript evidence without invented measurements r[octet.mutation_testing.boundary]

## Phase 3: Verification

- [ ] [parallel] Add positive clean-baseline, all-caught, survivor, timeout-review, and compatible-import fixtures r[octet.mutation_testing.verification]
- [ ] [parallel] Add negative failed-baseline, incomplete-inventory, duplicate-outcome, malformed, stale, unsafe-path, limit, and tool-failure fixtures r[octet.mutation_testing.verification]
- [ ] [parallel] Add policy fixtures that keep source outcomes separate from Octet gate decisions r[octet.mutation_testing.boundary]
- [ ] [parallel] Prove starter checks and unrelated evidence rails remain independent from cargo-mutants r[octet.mutation_testing.boundary]

## Phase 4: Documentation and closeout

- [ ] [parallel] Document profiles, baseline requirements, outcome classes, replay, diff retention, costs, and non-claims r[octet.mutation_testing.boundary]
- [ ] [serial] Run focused tests, Nickel freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.mutation_testing.verification]
- [ ] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.mutation_testing.profile]
