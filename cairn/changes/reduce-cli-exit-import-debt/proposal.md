# Reduce CLI exit import debt

## Why

Octet's canonical warning baseline prevents new stable identities, but it still accepts 1,114 `non_trait_imports` occurrences represented by 372 stable identities. The first reduction must prove a removals-only baseline refresh without aliases, exceptions, suppressions, or unrelated stable-ID churn.

Two plausible cohorts were rejected by old-baseline evidence. Qualifying four imports in `cargo-octet/build.rs` changed nine unrelated stable identities because other lints hash edited build-script snippets. Qualifying `std::fmt` in `domain_types.rs` changed two file-wide lint identities because those lints hash the full source. `cargo-octet/src/lib.rs` provides the surviving isolated cohort: its unique `std::process::ExitCode` import can be replaced in place, and the file has no whole-file lint identity.

## What Changes

- Replace the private `ExitCode` import with explicit `std::process::ExitCode` paths in the thin CLI shell.
- Prove against the old baseline that the change removes exactly one stable identity and introduces none.
- Refresh the reviewed baseline, documentation, and permanent Nix assertions to 2,669 warning occurrences, 1,819 stable identities, and 1,113 `non_trait_imports` occurrences.
- Keep the canonical warning-only exit contract, one-new-ID negative fixture, lint implementation, lint configuration, and CLI behavior unchanged.

## Impact

- **Files**: `cargo-octet/src/lib.rs`, `standards/self-check-warning-budget/octet-baseline.json`, `docs/self-check-warning-budget.md`, `flake.nix`, source-derived corpus fixtures, Cairn policy, and lifecycle artifacts.
- **Testing**: pre-change CLI and self-check gates; old-baseline stale/new assertions; focused CLI, baseline, source-corpus, and self-check checks; policy freshness, traceability, formatting, Clippy, Nextest, and the broad compatible-system flake gate.
