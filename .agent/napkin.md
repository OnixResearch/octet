# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|----------------|-------------------|
| 2026-03-23 | self | Tried dylint_linting v3 and v5 from crates.io — both broken on nightly 2026-03-21 (env_depinfo/file_depinfo removed from ParseSess) | Use git dep pointing to trailofbits/dylint `fix-nightly` branch until a new release lands |
| 2026-03-23 | self | Forgot `#![feature(rustc_private)]` and used `#[no_mangle]` instead of `#[unsafe(no_mangle)]` | Rust 2024 edition requires `unsafe()` wrapper on unsafe attrs; rustc_private feature gate is mandatory for compiler crate access |
| 2026-03-23 | self | Ran cargo test via pueue without nix devshell — missing pkg-config, openssl, LD_LIBRARY_PATH | Always use `nix develop -c bash -c '...'` for builds, or ensure env vars are set |
| 2026-03-23 | self | Tried setting RUSTUP_TOOLCHAIN in .cargo/config.toml [env] | Cargo explicitly forbids setting RUSTUP_TOOLCHAIN in [env] config. Set it in nix devShell or command line instead |
| 2026-03-23 | self | dylint_testing UI tests fail in nix — it shells out to `rustup which rustc` and builds dylint_driver which needs RUSTUP_TOOLCHAIN | Made UI test conditional: skip when rustup unavailable. Need proper nix-compatible test strategy for Phase 2 |
| 2026-03-23 | self | Used `cx.span_lint(...)` with closure — API changed to require `impl Diagnostic<'a, ()>` | Use `cx.opt_span_lint(lint, Some(span), DiagDecorator(\|diag\| { ... }))` — `DiagDecorator` wraps a closure into the `Diagnostic` trait |
| 2026-03-23 | self | `Item::ident` no longer exists in nightly 2026-03 | Ident moved into each `ItemKind` variant: `Fn` has `.ident`, `Struct(Ident, ..)`, `Enum(Ident, ..)`, `Trait` has `.ident`, `TyAlias` has `.ident` |
| 2026-03-23 | self | `TokenStream::trees()` doesn't exist | Use `TokenStream::iter()` which yields `&TokenTree` |

## User Preferences
- Uses openspec for structured change management
- Nix flakes for builds, nextest for testing
- Tiger Style coding philosophy (from TigerBeetle)
- Primary consumer of this lint crate is the aspen distributed systems project

## Patterns That Work
- `nix develop -c bash -c 'export RUSTUP_TOOLCHAIN=nightly && cargo test'` for running tests
- Flake devShell with `LD_LIBRARY_PATH` and `RUSTUP_TOOLCHAIN` env vars for rustc_private crates
- rustup shim via `pkgs.writeShellScriptBin` for nix environments
- Pointing to git branches for unreleased fixes via `{ git = "...", branch = "..." }` in Cargo.toml

## Patterns That Don't Work
- dylint_testing in pure nix — fundamentally tied to rustup (builds dylint_driver, uses `env -u RUSTUP_TOOLCHAIN`)
- Running cargo builds via pueue without the nix devshell env

## Domain Notes
- Project: `tigerstyle` — custom Clippy lints via dylint
- Phase 1 scaffolding complete. All 5 tasks done.
- 5 spec domains: safety, naming, structure, purity, concurrency
- 18 lints across 8 phases, using EarlyLintPass and LateLintPass
- Need to solve nix-compatible UI test strategy before Phase 2
- dylint PR #1942 "Fix nightly build" is the upstream fix — track for crates.io release
