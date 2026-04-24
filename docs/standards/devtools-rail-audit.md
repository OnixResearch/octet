# DevTools Rail Audit

This audit maps DevTools policy IDs to local-first enforcement rails after the
standards checker moved into the `tigerstyle` repo.

| Policy ID | Rail | Current enforcement |
|---|---|---|
| `tigerstyle.devtools.single-command-verification` | flake / local commands | `nix flake check`, README quick-start commands, and `docs/standards/policy-contract.md` |
| `tigerstyle.devtools.deny-warnings` | rustc / Clippy | crate-level `#![deny(clippy::all)]`, `#![deny(clippy::pedantic)]`, and `cargo clippy --workspace --all-targets -- -D warnings` |
| `tigerstyle.devtools.formatting` | rustfmt | `rustfmt.toml`, `cargo fmt --all --check`, and flake `fmt` check |
| `tigerstyle.devtools.documentation-gates` | rustc | crate-level `#![deny(missing_docs)]` for Rust crate roots |
| `tigerstyle.devtools.lint-suppressions` | `tigerstyle-standards` | suppression parser validates policy ID, source section, owner, rationale, review trigger, safer default, and narrow scope; flake `standards-fixtures` proves compliant and noncompliant fixture behavior |
| `tigerstyle.architecture.fcis-crate-boundary` | `tigerstyle-standards` | architecture fixtures prove the FCIS source-shape checker without coupling it to the Dylint crate |
| `tigerstyle.idioms.thin-forwarder-verifier` | Tiger Style lints | `.#cargo-tigerstyle` wrapper supplies `clang`, `mold`, `pkg-config`, and the pinned Rust toolchain |

## Local commands

```bash
nix develop path:. -c cargo fmt --all --check
nix develop path:. -c cargo clippy --workspace --all-targets -- -D warnings
nix develop path:. -c cargo test -p tigerstyle-standards
nix run .#cargo-tigerstyle -- check
nix build .#checks.x86_64-linux.policy-registry --no-link
nix build .#checks.x86_64-linux.standards-fixtures --no-link
nix run .#tigerstyle-policy-validate
nix run .#tigerstyle-standards -- --help
```
