## Phase 1: Policy and contracts

- [ ] [serial] Add the typed Nickel `cargo-fuzz-libfuzzer` profile with fixed command shape and policy-owned limits r[octet.dynamic_evidence.cargo_fuzz_profile]
- [ ] [serial] Add Rust-owned fuzz receipt DTOs and generated Nickel compatibility contracts r[octet.dynamic_evidence.cargo_fuzz_receipt]
- [ ] [serial] Register all new config and receipt families with explicit ownership and artifact freshness coverage r[octet.dynamic_evidence.cargo_fuzz_receipt]

## Phase 2: Core and shell

- [ ] [serial] Implement pure validation for target identity, budgets, corpus roots, dictionaries, crashes, reproducers, and deterministic diagnostics r[octet.dynamic_evidence.cargo_fuzz_receipt]
- [ ] [serial] Extend the dynamic-evidence shell with fixed cargo-fuzz argv, bounded artifact reads, and atomic receipt publication r[octet.dynamic_evidence.cargo_fuzz_profile]
- [ ] [serial] Preserve transcript evidence tiers and reject formal-proof or exhaustive-coverage promotion r[octet.dynamic_evidence.cargo_fuzz_boundary]

## Phase 3: Verification

- [ ] [parallel] Add positive clean-run, crash-discovery, and minimized-reproducer fixtures r[octet.dynamic_evidence.cargo_fuzz_verification]
- [ ] [parallel] Add negative fixtures for malformed summaries, stale hashes, unsafe paths, missing crash artifacts, limits, and command mismatches r[octet.dynamic_evidence.cargo_fuzz_verification]
- [ ] [parallel] Add Miri and Loom consumer fixtures that prove existing profile command isolation remains intact r[octet.dynamic_evidence.named_profile_regression]
- [ ] [parallel] Add a starter-path fixture that proves no dynamic tool is required without explicit selection r[octet.dynamic_evidence.cargo_fuzz_boundary]

## Phase 4: Documentation and closeout

- [ ] [parallel] Document profile selection, corpus privacy, crash replay, evidence fields, and non-claims r[octet.dynamic_evidence.cargo_fuzz_boundary]
- [ ] [serial] Run focused tests, Nickel freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.dynamic_evidence.cargo_fuzz_verification]
- [ ] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.dynamic_evidence.cargo_fuzz_profile]
