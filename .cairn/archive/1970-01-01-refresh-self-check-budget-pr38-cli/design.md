# Design: Restore owner-approved CLI source bytes and refresh PR #38 self-check warning budget

## Context

`cargo-octet/src/cli.rs` is one of 40 selected sources pinned in `reviewed-source-manifest.json` and `pr38-object-refresh-owner-approval.txt` at BLAKE3 `d181d2d9a38551c4aef2ad5ec7a7074ebc436846a5b7171f9cc1f2b0746ecd1d` (`113861db`). Refactoring `parse_requirements` in `38351dc9` broke byte parity for all 10 `reviewed-v2-source` Nix rails (`mir-supplement-inventory-gate`, etc.) and added a function object to the corpus. Meanwhile, `artifact.rs`, `prove.rs`, `requirements.rs`, and `requirements/evaluation/formal.rs` are not in the object corpus, so their `38351dc9` cleanups remain valid.

## Decisions

### Decision: Restore `cargo-octet/src/cli.rs` to `113861db` and waive `parse_requirements` `function_length`

**Choice:** Restore `cargo-octet/src/cli.rs` to `113861db` (`b3sum d181d2d9...`) and regenerate `standards/self-check-warning-budget/octet-baseline.json` via the packaged `cargo-octet` binary, accepting `parse_requirements` (`function_length`, 72 lines) and the updated `cli.rs` file length (5639 lines).

**Rationale:** Preserves exact byte identity with the owner-approved PR #38 object-corpus refresh (`pr38-object-refresh-owner-approval.txt`) without altering any sealed or owner-approved object-corpus artifacts, and matches the inline comment on `parse_requirements` explaining why flag matching stays linear in one function.

### Decision: Synchronize `flake.nix` totals and `docs/self-check-warning-budget.md`

**Choice:** Update `accepted_warning_total` (`4231`), `accepted_identity_total` (`3022`), `baseline_hash`, and baseline BLAKE3 in `flake.nix` and `docs/self-check-warning-budget.md` together with `octet-baseline.json`, and record all added/removed identities in `evidence/finding-delta.md`.

**Rationale:** Satisfies `r[octet.self_check.warning_budget_refresh]` and keeps `cargo-octet-self-check` strict zero-new/zero-stale verification intact.

## Risks / Trade-offs

- Adds one accepted `function_length` finding (`b3:3600e3c1b7e8cf1b`, 72 lines vs limit 70) to the advisory baseline, leaving a net -8/-8 warning/identity reduction relative to `113861db`.
- Local Nix and Cargo checks do not attest GitHub CI until PR #38 is pushed and CI completes.

## Verification

- `cargo fmt --check`, strict `cargo clippy`, and `cargo test --workspace` pass.
- `mir-supplement-inventory-gate`, `fresh-build-object-corpus-determinism`, `valence-object-evidence`, and `cargo-octet-self-check` pass.
- `core`, `mantle-aeneas-consumer-job`, `aeneas-lean-compatibility-matrix`, and full `nix flake check` pass.
- Cairn `validate`, `gate` (`proposal`, `design`, `tasks`), and `traceability coverage` pass.

## Non-claims

This is an advisory budget refresh, not a correctness, security, or release claim, and it does not attest GitHub CI.
