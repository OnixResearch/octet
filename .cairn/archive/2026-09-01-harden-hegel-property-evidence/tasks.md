## Phase 1: Profiles and contracts

- [x] [serial] Add typed Nickel Hegel producer profiles with fixed target selection, settings, limits, retention policy, and non-claims r[octet.dynamic_evidence.hegel_profiled_run]
- [x] [serial] Define the hardened producer-summary DTO and generated Nickel compatibility contract r[octet.dynamic_evidence.hegel_output_contract]
- [x] [serial] Register profile, summary, log, counterexample, receipt, and provenance artifact ownership and freshness checks r[octet.dynamic_evidence.hegel_measured_identity]

## Phase 2: Core and shell

- [x] [serial] Implement pure profile, summary, result, counterexample, limit, and deterministic-diagnostic validation r[octet.dynamic_evidence.hegel_output_contract]
- [x] [serial] Replace Hegel `--command` execution with fixed argv from the selected producer profile r[octet.dynamic_evidence.hegel_profiled_run]
- [x] [serial] Add the capability-rooted producer output path, bounded logs, timeout handling, and atomic publication shell r[octet.dynamic_evidence.hegel_profiled_run]
- [x] [serial] Remeasure Cargo lock, package, source, generator, profile, summary, and log identities before publication r[octet.dynamic_evidence.hegel_measured_identity]
- [x] [serial] Preserve bounded v1 import compatibility without inventing hardened measurements r[octet.dynamic_evidence.hegel_output_contract]

## Phase 3: Law catalog and verification

- [x] [parallel] Add a checked Hegel law catalog for selected pure cores with positive and negative generated properties r[octet.law_tested_deterministic_cores.hegel_catalog]
- [x] [parallel] Add fixture complements and replay cases for every selected law r[octet.law_tested_deterministic_cores.hegel_catalog]
- [x] [parallel] Add positive profiled-pass, counterexample, replay, and compatible-import fixtures r[octet.dynamic_evidence.hegel_hardening_verification]
- [x] [parallel] Add negative command, profile, schema, output, path, symlink, identity, limit, timeout, and overclaim fixtures r[octet.dynamic_evidence.hegel_hardening_verification]
- [x] [parallel] Prove starter checks and unrelated evidence rails remain independent from Hegel execution r[octet.dynamic_evidence.hegel_profiled_run]

## Phase 4: Documentation and closeout

- [x] [parallel] Document producer adapters, profiles, imports, replay, counterexample retention, evidence tiers, and non-claims r[octet.dynamic_evidence.hegel_output_contract]
- [x] [serial] Run focused Hegel laws, Cargo tests, Nickel freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.dynamic_evidence.hegel_hardening_verification]
- [x] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.dynamic_evidence.hegel_profiled_run]
