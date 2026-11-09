# Verification

Commands run on the reconciled main base:

- `cargo check --workspace --all-targets --locked`: passed.
- `cargo test -p octet --lib --locked`: 225 passed.
- `cargo test -p octet-lint-metadata --locked`: 22 passed.
- `cargo test -p octet-standards --lib storage_fault`: 4 passed.
- Full maintained UI rail (`cargo test -p octet --test ui ui -- --ignored`):
  passed, including `durability_handled`, `durability_recovery`,
  `durability_adapter`, `durability_discard`, `durability_flush_only`, and
  `durability_retry`.
- `cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings`: passed
  after collapsing three collapsible control-flow forms.
- `cargo fmt --check`: passed.
- Nickel typecheck/export freshness for `standards/durability-claims/catalog.ncl`,
  `profile.ncl`, and the storage-fault profile: passed.
- Regenerated `catalog.manifest.json` and `profile.manifest.json` with the
  checked `octet-standards nickel-export-manifest` command.
- `nix build path:$PWD#checks.x86_64-linux.durability-claims-policy` and
  `...#storage-fault-assurance` with local builders: both passed.
- `cairn validate --root .` and proposal/design/tasks gates: passed.

The lints remain inert without a consumer `durability_profile_path`; the
assurance validator performs no producer execution. No filesystem, device, or
runtime durability behavior is proven.
