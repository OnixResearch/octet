# Verification

Date: 2026-08-31

## Implemented boundary

- `standards/coverage/profiles.ncl` defines the checked `cargo-llvm-cov-octet-lib` profile.
- The generated profile registry has BLAKE3 identity `b3:e221f5d35528e6979bb44ab58cc5e486109799cf99fb80fc4d85ab77a7082634`.
- The profile pins cargo-llvm-cov 0.8.3 through nixpkgs revision `b3d51a0365f6695e7dd5cdf3e180604530ed33b4`.
- The profile fixes Rust, LLVM tools, package scope, features, tests, metrics, arguments, thresholds, comparison policy, paths, limits, and non-claims.
- Nix exposes the provider as `cargo-llvm-cov-provider` and includes it in the wrapped Octet runtime.
- `cargo-octet/src/coverage/` separates native parsing, exact-count comparison, request assembly, shell effects, and tests.
- Native normalization reads covered and total counts. It does not trust producer percentages.
- Display percentages use integer basis points with floor rounding.
- Threshold and regression decisions use exact integer cross multiplication.
- Comparisons require matching profile, provider, toolchain, source scope, path policy, and source-manifest identity.
- The equal-denominator policy rejects total-count changes and preserves both count pairs.
- The shell rejects arbitrary commands and clears the child environment.
- The Nix composition supplies the exact Cargo path, provider path, vendor configuration, and runtime library path.
- Bounded Exec owns deadlines, output limits, process-group teardown, and outcome classification.
- Octet consumes `OnixResearch/bounded-exec` at revision `44eb74eb639194665ee820562d5b2ce2c3ff6a94` under the owner reuse grant.
- The import path reads current, baseline, profile, and source-manifest files through one capability root.
- Existing normalized v1 imports remain legacy transcript evidence when no hardened profile is selected.

## Receipt and artifact evidence

- Rust owns native-envelope, count, comparison, and receipt DTOs.
- The generated Nickel contract includes `CoverageBaselineReceiptDetailsV1`.
- The policy registry records profiles, source manifests, raw reports, envelopes, comparisons, receipts, logs, and provenance ownership.
- The bundle retains the source manifest, exact current report, current envelope, accepted baseline envelope, and comparison projection.
- Artifact validation remeasures every retained file and recalculates the comparison.
- The Valence export preserves typed coverage details and transcript roles.
- The export does not create a formal-proof role.

## Positive and negative evidence

- Positive fixtures cover stable counts, improved counts, thresholds, native process output, import publication, and legacy imports.
- Negative fixtures cover regressions, denominator changes, percentage-only input, malformed reports, unsafe paths, stale envelopes, and stale source manifests.
- Shell tests reject output overflow and nonzero provider exits.
- Input tests reject stale profile bytes, multiple authority modes, arbitrary commands, and missing bounded inputs.
- Bundle tests reject changed raw report bytes after publication.
- Exact-precision tests prove that two of three produces 6,666 basis points without floating-point policy.

## Passing commands

- `nix develop --builders '' -c cargo test -p cargo-octet --lib coverage::` passed 6 tests.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib profiled_coverage` passed 3 tests.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib evidence_rails::tests` passed 42 tests.
- The focused evidence-contract, fixture-matrix, artifact-verification, and Valence-export tests passed.
- Library and test Clippy gates passed with all features and `-D warnings`.
- Nickel 1.17 typecheck and deterministic export passed.
- The `coverage-profiles`, `policy-registry`, and `cairn-traceability` Nix checks passed with local builders.
- The new `cargo-octet/src/coverage/` modules report zero Octet findings.
- Cairn validation and proposal, design, and tasks gates passed with the canonical workspace policy.
- `git diff --cached --check` passed.

## Live provider probe

An isolated cargo-llvm-cov 0.8.3 probe ran 12 `octet-lint-metadata` library tests. The native report matched schema version 2.0.1.

The probe produced safe relative paths and integer covered and total counts. The report BLAKE3 digest was `7e919e68fabff921d05c632073266ea7b8c2d85a100df3964802eab67b639277`.

The first probe reused the shared Cargo target and found one stale Lattice trybuild manifest. The isolated target removed that unrelated environment input.

## Inherited blockers

The broad cargo-octet library suite reports 425 passed, two failed, and five ignored tests. Both failures use the pre-existing Nickel identity mismatch.

The same failures existed on untouched `origin/main` before this change. This change does not replace a baseline or suppress a finding.

The broad self-check also retains its pre-existing warning-budget drift.

## Non-claims

Coverage counts show instrumented execution only. They do not prove behavioral correctness, path correctness, test completeness, or formal proof.

A source-manifest identity binds supplied bytes only. It does not prove complete source inclusion.

The receipt does not prove whole-program correctness or release eligibility.
