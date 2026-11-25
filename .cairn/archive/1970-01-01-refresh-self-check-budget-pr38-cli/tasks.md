# Tasks: Restore owner-approved CLI source bytes and refresh PR #38 self-check warning budget

## Phase 1: Restore CLI source and refresh self-check baseline

- [x] [serial] Restore `cargo-octet/src/cli.rs` to `113861db` (`b3sum d181d2d9a38551c4aef2ad5ec7a7074ebc436846a5b7171f9cc1f2b0746ecd1d`). r[octet.self_check.warning_budget_refresh]
- [x] [serial] Regenerate `standards/self-check-warning-budget/octet-baseline.json` using the packaged `cargo-octet check --output-format json --write-baseline` and record the added/removed identities in `evidence/finding-delta.md`. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Synchronize `flake.nix` (`accepted_warning_total=4231`, `accepted_identity_total=3022`) and `docs/self-check-warning-budget.md` with the refreshed baseline. r[octet.self_check.warning_budget_refresh]

## Phase 2: Verification and archive

- [x] [serial] Verify `cargo fmt --check`, strict `cargo clippy`, `cargo test --workspace`, object-corpus gates (`mir-supplement-inventory-gate`, `fresh-build-object-corpus-determinism`, `valence-object-evidence`), `cargo-octet-self-check`, `core`, `mantle-aeneas-consumer-job`, `aeneas-lean-compatibility-matrix`, and full `nix flake check`. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Run Cairn `validate`, `gate` (`proposal`, `design`, `tasks`), `traceability coverage`, and archive the change with recorded evidence. r[octet.self_check.warning_budget_refresh]

## Verification Coverage

- `Scenario: Refreshed baseline has no stale or new findings` -> `cargo-octet-self-check` derivation pass
- `Scenario: Counts match the baseline` -> `evidence/baseline-refresh.txt` and `cargo-octet-self-check`
- `Scenario: Added identities are recorded` -> `evidence/finding-delta.md`
- `Scenario: Warning budget remains advisory` -> proposal and design non-claims
