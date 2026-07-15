## Phase 1: Typed policy and pure core

- [x] [serial] Add the Nickel Clippy restriction catalog, contracts, generated JSON, valid/invalid fixtures, policy-family registration, standalone packaging inputs, and deterministic Nix freshness check. r[octet.external_lint.clippy_restriction_profiles.catalog] r[octet.external_lint.clippy_restriction_profiles.catalog.valid] r[octet.external_lint.clippy_restriction_profiles.catalog.invalid]
- [x] [serial] Add the pure Rust catalog parser, deterministic diagnostics, profile selection, supported-lint validation, BLAKE3 selected-profile identity, and fixed command construction. r[octet.external_lint.clippy_restriction_profiles.catalog] r[octet.external_lint.clippy_restriction_profiles.run.profiled]

## Phase 2: Evidence-rail shell and receipt binding

- [x] [serial] Wire profiled Clippy run mode through the existing evidence-rail shell, reject arbitrary command authority, parse stdout-only JSON, preserve unprofiled compatibility, and fail closed on nonzero tool exits. r[octet.external_lint.clippy_restriction_profiles.run] r[octet.external_lint.clippy_restriction_profiles.run.profiled] r[octet.external_lint.clippy_restriction_profiles.run.compatibility] r[octet.external_lint.clippy_restriction_profiles.receipt.tool_failure]
- [x] [serial] Bind full config and selected-profile BLAKE3 identities plus exact lint posture into receipt details and reject missing, malformed, or command-drifted profiled receipts. r[octet.external_lint.clippy_restriction_profiles.receipt] r[octet.external_lint.clippy_restriction_profiles.receipt.policy_bound]

## Phase 3: Verification and documentation

- [x] [parallel] Add positive and negative unit/integration coverage for valid and malformed catalogs, clean and violating Clippy workspaces, arbitrary-command rejection, policy identity, nonzero exits, and receipt tampering. r[octet.external_lint.clippy_restriction_profiles.verification]
- [x] [parallel] Document profile intent, composition with Octet strict profiles, invocation, local exception posture, toolchain drift, and explicit non-claims in README, consumer guidance, Clippy profile guidance, CLI help, and changelog. r[octet.external_lint.clippy_restriction_profiles.boundaries] r[octet.external_lint.clippy_restriction_profiles.boundaries.no_blanket]
- [x] [serial] Run focused Rust tests, integration fixtures, Nickel freshness/fixture checks, formatting, policy validation, and Cairn validation/gates; record exact outcomes under the change evidence directory. r[octet.external_lint.clippy_restriction_profiles.verification]
