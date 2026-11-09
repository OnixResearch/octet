# Verification

Target directory: `/home/brittonr/.cache/octet-work-kamacite/target` with
`CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_INCREMENTAL=0`.

## Rust

- `cargo check --workspace --all-targets`: PASS.
- `cargo test -p cargo-octet kamacite_effect_closure`: 18 passed.
- `cargo test -p cargo-octet admit::`: 13 passed.
- `cargo test -p cargo-octet cli::`: 43 passed.
- `cargo test -p cargo-octet effect_closure`: 64 passed.
- `cargo test -p octet --test consumer_integration effect_closure_admission_stops_after_failed_direct_check`: 1 passed.
- `cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings`: PASS.
- `cargo fmt --check`: PASS.

## Nickel and policy freshness

- `nickel typecheck standards/effect-closure/kamacite-profile.ncl` and export
  `cmp` against `standards/effect-closure/generated/kamacite-effect-profile.json`: PASS.
- `nickel typecheck standards/effect-closure/fixtures/kamacite/effect-summary.ncl`
  and export `cmp` against `standards/effect-closure/fixtures/kamacite/effect-summary.json`: PASS.
- All `fixtures/kamacite/valid/*.ncl` and `fixtures/kamacite/rejected/*.ncl`
  export successfully; all `fixtures/kamacite/invalid/*.ncl` fail export: PASS.
- `nickel typecheck standards/policies/registry.ncl` and export: PASS; the new
  `octet.effect-closure-kamacite-profile`, `octet.kamacite-effect-comparison`,
  and `octet.kamacite-effect-handoff` families are present.

## Lifecycle

- `cairn validate --root .`: PASS.
- `cairn gate proposal bind-effect-closure-to-kamacite --root .`: PASS.
- `cairn gate design bind-effect-closure-to-kamacite --root .`: PASS.
- `cairn gate tasks bind-effect-closure-to-kamacite --root .`: PASS.

## Nix

`nix build .#checks.x86_64-linux.effect-closure-kamacite-profile --no-link -L
--option builders '' --option post-build-hook ''`: PASS (exit 0). The default
host build-hook configuration failed before the build with a remote gRPC lock
path `File name too long`; disabling remote builders and the post-build hook
let the local check run.

No UI fixture changed in this change; the existing UI rail was not rerun.
