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
| 2026-03-23 | self | `cx.tcx.hir()` removed, `item.attrs` gone from HIR Item | Use `cx.tcx.hir_parent_iter()` and `cx.tcx.hir_attrs(hir_id)` for attr access. HIR Item fields: owner_id, kind, span, vis_span |
| 2026-04-13 | self | Used `rg` tool with shell-style alternation pattern containing spaces and `|`, which the wrapper passed through incorrectly | For alternation-heavy searches, use `bash` with explicit quoting or avoid spaces in the raw `rg` pattern |
| 2026-04-13 | self | Aspen-side Dylint rollout under nix-managed nightly failed until the check wrapped `rustup`/`cargo`/`dylint-link` and patched `dylint_driver` through a temporary `CARGO_HOME/config.toml` | For downstream integration, use a wrapper script like `scripts/tigerstyle-check.sh`; do not assume plain `cargo dylint` works under nix-managed toolchains |
| 2026-04-13 | self | Saved `git status --short` as a cleanliness artifact, but clean repo produced a zero-byte file that was not self-describing in review | When a status command may emit nothing, save the command line and an explicit `(no output; repo clean)` note in the artifact |
| 2026-04-13 | self | Saved targeted `cargo clippy` evidence from a warm cache, which only showed resolver warnings and `Finished` without proving which crates were checked | For reviewable clippy evidence, save the command line and use a fresh `CARGO_TARGET_DIR` so the artifact includes `Checking aspen-*` lines for the targeted crates |
| 2026-04-14 | self | Treated Aspen HEAD `scripts/tigerstyle-check.sh` as if it were the original phase-1 starter path; Aspen has already expanded scope/lints past the archived starter rollout | For Aspen starter-path reconstruction, use `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` evidence, not current `/home/brittonr/git/aspen` scripts |

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
- Aspen phase-1 rollout reference is the archived change `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`; Aspen HEAD may already be on a later rollout profile
- Phase 1 scaffolding complete. All 5 tasks done.
- 5 spec domains: safety, naming, structure, purity, concurrency
- 18 lints across 8 phases, using EarlyLintPass and LateLintPass
- Need to solve nix-compatible UI test strategy before Phase 2
- dylint PR #1942 "Fix nightly build" is the upstream fix — track for crates.io release
