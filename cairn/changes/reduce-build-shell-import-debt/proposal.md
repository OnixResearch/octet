# Reduce build-shell import debt

## Why

Octet's canonical warning baseline now prevents new stable identities, but it still accepts 1,114 `non_trait_imports` occurrences represented by 372 stable identities. The first reduction should be small enough to prove the stale-ID workflow without hiding debt through aliases, exceptions, suppressions, or a broad source rewrite.

`cargo-octet/build.rs` is an isolated imperative build shell with exactly four non-trait imports: `std::env`, `std::fs`, `std::path::Path`, and `std::path::PathBuf`. Its `std::fmt::Write as _` import is a trait-resolution import and is correctly outside the lint.

## What Changes

- Replace the four build-shell non-trait imports with explicit owner-qualified use sites while retaining the trait import.
- Prove against the old baseline that the change removes exactly four stable identities and introduces none.
- Refresh the reviewed baseline, documentation, and permanent Nix assertions to 2,666 warning occurrences, 1,816 stable identities, and 1,110 `non_trait_imports` occurrences.
- Keep the canonical warning-only exit contract, one-new-ID negative fixture, lint implementation, lint configuration, and build output behavior unchanged.

## Impact

- **Files**: `cargo-octet/build.rs`, `standards/self-check-warning-budget/octet-baseline.json`, `docs/self-check-warning-budget.md`, `flake.nix`, Cairn policy and lifecycle artifacts.
- **Testing**: pre-change embedded-source tests and self-check gate; old-baseline stale/new assertions; focused build, baseline, and self-check checks; policy freshness, traceability, formatting, Clippy, Nextest, and the broad compatible-system flake gate.
