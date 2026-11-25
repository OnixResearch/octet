# Commands and results

Verification executed on `aspen1` inside `nix develop --option builders ""` with `CARGO_TARGET_DIR=/dev/shm/kiln-swarm/octet-worker/target` and `TMPDIR=/dev/shm/kiln-swarm/octet-worker/tmp`:

| Command | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic -D clippy::dbg_macro -D clippy::mem_forget -D clippy::undocumented_unsafe_blocks -D clippy::multiple_unsafe_ops_per_block` | exit 0 |
| `cargo test --workspace` | exit 0 |
| packaged `cargo-octet check --output-format json --write-baseline` | exit 0, 3,022 identities / 4,231 warnings (recorded in `baseline-refresh.txt`) |
| `nix build --option builders "" .#checks.x86_64-linux.mir-supplement-inventory-gate .#checks.x86_64-linux.fresh-build-object-corpus-determinism .#checks.x86_64-linux.valence-object-evidence .#checks.x86_64-linux.cargo-octet-self-check --no-link` | exit 0 (`reviewed-v2-source: ADMITTED`) |
| `cairn validate`, `cairn gate {proposal,design,tasks}`, `cairn traceability coverage` | exit 0 (recorded in `cairn-gates.txt`) |
