# DevTools Rail Audit

This audit maps DevTools policy IDs to local-first enforcement rails after the
standards checker moved into the `slotcar` repo.

| Policy ID | Rail | Current enforcement |
|---|---|---|
| `slotcar.devtools.single-command-verification` | flake / local commands | `nix flake check`, README quick-start commands, and `docs/standards/policy-contract.md` |
| `slotcar.devtools.deny-warnings` | rustc / Clippy | crate-level `#![deny(clippy::all)]`, `#![deny(clippy::pedantic)]`, and `cargo clippy --workspace --all-targets -- -D warnings` |
| `slotcar.devtools.formatting` | rustfmt | `rustfmt.toml`, `cargo fmt --all --check`, and flake `fmt` check |
| `slotcar.devtools.documentation-gates` | rustc | crate-level `#![deny(missing_docs)]` for Rust crate roots |
| `slotcar.devtools.lint-suppressions` | `slotcar-standards` | suppression parser validates policy ID, source section, owner, rationale, review trigger, safer default, and narrow scope; flake `standards-fixtures` proves compliant and noncompliant fixture behavior |
| `slotcar.architecture.fcis-crate-boundary` | `slotcar-standards` | architecture fixtures prove the FCIS source-shape checker without coupling it to the Dylint crate |
| `slotcar.idioms.thin-forwarder-verifier` | Tiger Style lints | `.#cargo-slotcar` wrapper supplies `clang`, `mold`, `pkg-config`, and the pinned Rust toolchain |

## Local commands

```bash
nix develop path:. -c cargo fmt --all --check
nix develop path:. -c cargo clippy --workspace --all-targets -- -D warnings
nix develop path:. -c cargo test -p slotcar-standards
nix run .#cargo-slotcar -- check
nix build .#checks.x86_64-linux.policy-registry --no-link
nix build .#checks.x86_64-linux.standards-fixtures --no-link
nix run .#slotcar-policy-validate
nix run .#slotcar-standards -- --help
```
