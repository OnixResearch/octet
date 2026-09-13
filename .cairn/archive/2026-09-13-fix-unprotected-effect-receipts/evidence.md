# Provider repair evidence

## Scope and results

Base revision: `0b1ca30a7e86bb38369bdf2b8a851772a3b9f213`.
Only the authorization-identity precondition, its tests, and related documentation change.
Receipt fields, schema, compiler collection, policy declarations, lint catalog, and generated contracts do not change.

| Check | Observation |
|---|---|
| Existing architecture-gate baseline | 22 tests passed |
| New unprotected positive before the fix | Failed with `project architecture receipt lacks authorized-effect fact or type identities` |
| Updated architecture-gate suite | 27 tests passed |
| Standards package, all targets and features | 174 unit tests and 24 integration tests passed |
| Formatting | Passed, including the Nix formatting check |
| Standards-package Clippy, `-D warnings` | Passed |
| Nix `project-architecture-policy` | Passed |
| Nix `cargo-octet` package | Built successfully |
| Noble compatibility experiment | Receipt now self-verifies; two separate test-source role conflicts remain visible and blocking |

The Noble experiment is not Noble acceptance. It used a candidate build before publication.
No unprotected output declaration was removed to make that experiment pass.
The two remaining findings are `source-role-unit-mismatch` for Noble's kernel and CLI integration-test scopes.

## Replay

Run from the provider change worktree. These paths keep data outside disposable worktrees and avoid the full `/tmp` filesystem.

```sh
export TMPDIR=/home/brittonr/git/noble/.pi/m1-octet-repair/tmp
export CARGO_HOME=/home/brittonr/git/noble/.pi/m1-octet-repair/cargo
export CARGO_TARGET_DIR=/home/brittonr/git/noble/.pi/m1-octet-repair/target
unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
nix develop -c sh -c '
  cp "$OCTET_COVERAGE_CARGO_CONFIG" "$CARGO_HOME/config.toml" &&
  cargo test -p octet-standards --lib project_architecture_gate::tests --locked --offline -- --nocapture &&
  cargo test -p octet-standards --all-targets --all-features --locked --offline &&
  cargo fmt --all -- --check &&
  cargo clippy -p octet-standards --all-targets --all-features --locked --offline -- -D warnings
'
nix build .#cargo-octet .#checks.x86_64-linux.fmt .#checks.x86_64-linux.project-architecture-policy -L
```

`OCTET_COVERAGE_CARGO_CONFIG` supplies Nix's locked vendor sources. Its use here is not coverage evidence.
Without that config, Cargo attempted an unavailable HTTPS authentication route for private dependencies.
The first combined test command passed its tests but stopped at formatting's dependency resolution.
Installing the same Nix config in the private Cargo home resolved that operational failure.

## Supplementary check boundary

An additional `-W clippy::pedantic` run fails with 21 findings in unchanged `octet-architecture-ir` library code.
A separate run reproduced those findings after `git diff --exit-code origin/main -- crates/octet-architecture-ir` established no source changes there.
This supplementary baseline does not block the narrow receipt repair's declared `-D warnings` package gate.
It does block any claim that the full repository's pedantic gate passed.
No finding was suppressed or baseline file changed. Full external proof and release rails were not executed.

## Retention and traceability

Exact commands, exit statuses, diagnostics, and build output are retained in the main Noble repository's `.pi/m1-octet-repair/provider-receipts.log`.
The relevant Pueue tasks were 1863 (baseline), 1866 (red regression), 1872 (tests), 1874/1877 (supplementary baseline), 1876 (strict Clippy), 1873 (Nix), and 1879 (Noble experiment).
The change's scenario table maps all three scenarios to the corresponding Rust tests.
These are local observed test and build results, not independently authenticated release or semantic proof evidence.
