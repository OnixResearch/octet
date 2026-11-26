# Proposal: Restore owner-approved CLI source bytes and refresh PR #38 self-check warning budget

## Why

Commit `38351dc9` split `requirements_subcommand` out of `parse_requirements` in `cargo-octet/src/cli.rs` after commits `8ffb83c7`/`113861db` had already frozen `cli.rs` (`d181d2d9a38551c4aef2ad5ec7a7074ebc436846a5b7171f9cc1f2b0746ecd1d`) under the owner-approved PR #38 object-corpus refresh (`pr38-object-refresh-owner-approval.txt`). That post-approval edit caused `checks.x86_64-linux.mir-supplement-inventory-gate` (and the sibling `reviewed-v2-source` object rails) to reject `cargo-octet/src/cli.rs` for selected source byte drift.

## What Changes

- Restore `cargo-octet/src/cli.rs` to its owner-approved `113861db` bytes (`d181d2d9a38551c4aef2ad5ec7a7074ebc436846a5b7171f9cc1f2b0746ecd1d`), keeping all non-corpus cleanups from `38351dc9` (`artifact.rs`, `prove.rs`, `requirements.rs`, `requirements/evaluation/formal.rs`).
- Accept `parse_requirements` `function_length` (72 lines vs limit 70) in the advisory self-check baseline and regenerate `standards/self-check-warning-budget/octet-baseline.json` using the packaged `cargo-octet check --output-format json --write-baseline` (`3,022` unique stable identities, `4,231` warning occurrences).
- Update `flake.nix` (`accepted_warning_total=4231`, `accepted_identity_total=3022`) and `docs/self-check-warning-budget.md` stored-budget paragraph, and record the exact finding delta in `evidence/`.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `standards/self-check-warning-budget/octet-baseline.json`, `flake.nix`, `docs/self-check-warning-budget.md`
- **Testing**: `cargo fmt --check`, strict `cargo clippy --workspace --all-targets`, `cargo test --workspace`, `mir-supplement-inventory-gate`, `fresh-build-object-corpus-determinism`, `valence-object-evidence`, `cargo-octet-self-check`, `core`, `mantle-aeneas-consumer-job`, `aeneas-lean-compatibility-matrix`, and full `nix flake check` (`verify`).

## Non-claims

The warning budget is an advisory regression baseline, not correctness or release evidence. The refresh records the current reviewed set; it does not prove accepted warnings are harmless or complete, and it enables or disables no lint. Local Nix gates do not attest GitHub CI or release qualification.
