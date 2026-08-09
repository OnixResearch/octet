# Validation evidence

Date: 2026-08-09

## Admitted input

- Cairn revision: `695124d459574ba7aeba6097310d237f393c243c`
- The targeted Nix lock update was generated with `nix flake lock --update-input cairn`.
- The resulting root `cairn` lock node resolves to the admitted revision.

## Lifecycle migration

- Preview plan hash: `da12900602af6a975219ee5b38471961688b0c7b3c7c6d78baab028c5f0089b5`
- The migration receipt is retained in `evidence/migration-receipt.json`.
- A sorted BLAKE3 manifest comparison found byte parity between the legacy `cairn/` tree and native `.cairn/` tree before legacy removal.
- Maintained Octet paths now select `.cairn/`. Synthetic migration fixtures, historical receipt text, and external repository references retain their original paths.

## Policy rebase

Octet's policy now overlays its traceability profile and custom Valence-provenance receipt schema onto Cairn's schema-compatible upstream policy. The checked-in export contains the `octet-default` profile, native `.cairn/specs/` requirement roots, and the custom receipt command.

## Passing checks

- Cairn policy export and freshness check: pass.
- `checks.x86_64-linux.cairn-traceability`: pass.
- `checks.x86_64-linux.policy-registry`: pass.
- `checks.x86_64-linux.warning-clean`: pass after the Nix source filter admitted native `.cairn/` evidence.
- `cargo test -p octet-lint-metadata frozen_pilot_receipts_are_valid_and_reviewed`: pass.
- `cargo test --manifest-path crates/valence-core/Cargo.toml evidence_codebase_index`: 11 pass.
- `git diff --check`: pass.

## Inherited blockers

The pre-change `origin/main` baseline is not fully green.

- `checks.x86_64-linux.cargo-octet-self-check` exits nonzero without a diagnostic. This failure was reproduced before the dependency update.
- `cargo test -p octet catalog_docs_match_inventory_counts_defaults_and_rows` fails because the unchanged README does not match the current lint-catalog total and namespace expectations. The same failure was reproduced in a detached clean worktree at baseline revision `d87153a1`.

The final `nix flake check path:$PWD -L` reaches the same unchanged lint-catalog documentation failure after all migration-specific failures are corrected. This change does not repair those independent baseline defects.

## Boundary

This evidence covers the Cairn input update, lifecycle-store migration, policy-schema rebase, and listed checks. It does not claim that Octet's inherited full-suite blockers are fixed.
