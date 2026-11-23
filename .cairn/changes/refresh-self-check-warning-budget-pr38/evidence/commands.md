# Commands and results

Environment:

```bash
export PATH=/nix/store/2msw5nclhpsm80z65dqmjiv8kkras989-clang-wrapper-21.1.8/bin:$PATH
export CARGO_TARGET_DIR=/tmp/octet-check-target RUSTC_WRAPPER= CARGO_INCREMENTAL=0
```

| Command | Result |
|---|---|
| `cargo check -p cargo-octet --lib` | exit 0, no warnings |
| `rustfmt --edition 2024 --check` on the five touched files | clean |
| `cargo clippy -p cargo-octet --all-targets -- -D warnings -W clippy::pedantic -D clippy::dbg_macro -D clippy::mem_forget -D clippy::undocumented_unsafe_blocks -D clippy::multiple_unsafe_ops_per_block` | exit 0, `Finished dev profile` |
| `cargo test -p cargo-octet --lib -- prove:: artifact:: requirements::` | exit 0, 138 passed / 0 failed / 1 ignored (recorded in `tests.txt`) |
| packaged `cargo-octet check --output-format json --write-baseline` | exit 0, 3,021 identities / 4,230 warnings (recorded in `baseline-refresh.txt`) |
| `nix build -L --no-link path:$PWD#checks.x86_64-linux.cargo-octet-self-check` | exit 0 (recorded in `nix-self-check.log`) |
| `cairn validate --root . --policy cairn-policy/generated/cairn-policy.json` & proposal/design/tasks gates | exit 0, 0 issues (recorded in `cairn-gates.txt`) |

The packaged runner used for the baseline refresh is the same derivation the
Nix gate builds; the refresh ran against the edited worktree sources.
