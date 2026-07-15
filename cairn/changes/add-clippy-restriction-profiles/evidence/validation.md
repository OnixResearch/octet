# Validation evidence

Date: 2026-07-15

## Baseline

- `cargo test --locked -p cargo-octet --lib evidence_rails::tests` — passed 28 existing evidence-rail tests before core changes.
- The unmodified development shell initially failed because host Nix configuration referenced missing `/run/secrets/vars/nix-signing-key/key`; validation used `--option secret-key-files ''` or focused standalone Nix shells without changing repository policy.

## Policy and generated artifacts

- Standalone Nickel typecheck/export plus valid and invalid fixture evaluation — passed.
- `nix build .#checks.x86_64-linux.clippy-restriction-profiles --no-link -L --option secret-key-files ''` — passed deterministic export freshness and valid/invalid fixture checks.
- `nix run --option secret-key-files '' .#octet-policy-validate` — passed.

## Rust and integration checks

- `cargo test --locked -p cargo-octet --lib clippy` — passed 8 focused positive/negative unit tests.
- `cargo test --locked -p cargo-octet --lib evidence_rails::tests` — passed 31 evidence-rail tests after implementation.
- `cargo test --locked -p cargo-octet --test clippy_restriction_profiles -- --nocapture` — passed both the clean and violating real-Clippy workspace tests; the clean artifact bundle also passed `cargo octet artifact verify`.
- `cargo clippy --locked -p cargo-octet --all-targets -- -D warnings` under the pinned `nightly-2026-03-21` toolchain — passed.
- `cargo fmt --all -- --check` under the pinned toolchain — passed.
- `nixfmt --check flake.nix` — passed.

## Policy-owner dogfood

- `nix build .#checks.x86_64-linux.clippy --no-link -L --option secret-key-files ''` — passed while deriving the exact `production-safety` lint level and lint IDs from generated catalog JSON.
- A policy-bound `production-safety` evidence run over the Octet workspace passed with zero findings and tool exit code zero.
- `cargo-octet artifact verify` reported `evidence/octet-production-safety` valid.
- The receipt binds catalog identity `b3:125e578fa37bfcb314108c80992df302b3dd2647df0f0d042c3ac205b82010cd` and selected profile identity `b3:249384f4b01c9e222c172b9062252e4d195323c9ef416449f501390360b5d086`.

## Lifecycle

- Proposal, design, and tasks gates — passed under `octet-default` with no issues.
- `cairn validate --root . --policy cairn-policy/generated/cairn-policy.json` — passed with one active change, 67 accepted specs, and no issues.

## Environment notes

Nix attempted the configured remote builder during some checks, reported it unavailable, and completed derivations locally. One development-shell evaluation also reported an unavailable Git HTTPS helper while using the already pinned cached Dylint revision; the shell and checks completed successfully.
