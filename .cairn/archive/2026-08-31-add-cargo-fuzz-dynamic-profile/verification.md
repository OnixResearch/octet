# Verification

Date: 2026-08-31

## Implemented boundary

- `templates/dynamic-evidence-profiles.ncl` defines one typed `cargo-fuzz-libfuzzer` profile.
- `dynamic-evidence-profiles.json` has exact BLAKE3 identity `b3:7ad698ddebaaac2640a51e5f58871ac026b4b32653fb95bf971ec491560b48d6`.
- The profile fixes the target, engine, nightly toolchain, sanitizer, argv, seed, run budget, time budget, corpus paths, artifact paths, and limits.
- `cargo-octet/src/cargo/fuzz/` separates receipt DTOs, pure validation, process and filesystem effects, and tests.
- Run mode rejects arbitrary `--command` input. It snapshots bounded corpus and crash artifacts, runs fixed argv, and emits one typed receipt.
- Import mode reads a bounded receipt and publishes it without cargo-fuzz installed.
- Evidence receipt replacement uses a capability-rooted atomic stage and rename.
- Clean observations remain transcript assertions. Crash and tool-failure observations fail closed.
- Valence export preserves cargo-fuzz details and never emits a formal-proof edge.

## Positive and negative evidence

- Clean, crash, and malformed JSON fixtures are checked in under `cargo-octet/tests/fixtures/evidence-rails/`.
- Nickel accepts the valid receipt contract fixture and rejects the wrong-profile fixture.
- Tests reject stale corpus roots, unsafe paths, bad BLAKE3 identities, excessive budgets, missing crash roles, bad replay argv, unsupported tiers, and proof claims.
- Tests preserve Miri and Loom command isolation.
- The starter CLI path remains independent from all dynamic tools.
- The import publication test succeeds without invoking cargo-fuzz.
- Shell tests hash bounded files, reject symlinks, classify failures, and reject excessive logs.

## Passing commands

- `nix develop --builders '' -c cargo test -p cargo-octet --lib cargo::fuzz` — 8 passed.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib evidence_rails::tests::cargo_fuzz` — 4 passed.
- Focused evidence-contract, artifact-export, fixture-matrix, and capability-publication tests passed.
- `nix develop --builders '' -c cargo clippy -p cargo-octet --lib --all-features -- -D warnings` — passed.
- `nix develop --builders '' -c cargo clippy -p cargo-octet --tests --all-features -- -D warnings` — passed before the final shell-only addition; the final library Clippy gate passed.
- `nix build .#checks.x86_64-linux.dynamic-evidence-profiles --no-link -L --builders ''` — passed.
- `nix build .#checks.x86_64-linux.policy-registry .#checks.x86_64-linux.cairn-traceability --no-link -L --builders ''` — passed.
- Nickel typecheck, export freshness, generated contract freshness, and positive and negative Nickel fixtures passed.
- Focused `cargo octet check -p cargo-octet -- --lib` reports no findings in `cargo-octet/src/cargo/fuzz/`.

## Inherited blockers

The broad library test runs 412 tests before this change’s final shell tests. It reports 406 passed, four ignored provider tests, and two failures. Both failures use the pre-existing test literal `nickel-test-identity` instead of the accepted Nickel 1.17 identity. The same two failures reproduce on detached `origin/main`.

The canonical `cargo-octet-self-check` Nix derivation is also broken on detached `origin/main`. Both the unchanged branch and this change report 89 findings, 81 new identities, 1,813 stale identities, and integration-failure status. This change does not alter the baseline or suppress those failures.

A wider cargo-octet scan reports 1,555 findings after this change versus 1,557 on detached `origin/main`. New cargo-fuzz modules are finding-free. Existing file-level identities change when their receipt fields and integration functions change; no baseline was replaced.

## Non-claims

Cargo-fuzz evidence covers one target, seed, corpus, sanitizer, and bounded run. It does not prove exhaustive coverage, memory safety, panic freedom, whole-program correctness, or release eligibility.
