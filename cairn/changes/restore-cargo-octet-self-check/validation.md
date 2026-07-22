# Validation

## Outcome

The canonical repository command is restored. Before implementation, `nix run .#cargo-octet -- check` exited `2` at metadata admission because root `Cargo.toml` had no `[workspace.metadata.octet]`. The root now declares exact no-argument policy `default_scope = ["--workspace"]` and `cargo_check_args = ["--lib"]`; the loader and its fail-closed consumer behavior were not weakened.

The first post-metadata execution exposed a second real packaging defect: root `.cargo/config.toml` selects `clang` with `-fuse-ld=mold`, but those executables were available while building the package and absent from the wrapped runtime path. `cargoOctetRuntimePath` now includes both pinned tools. The final exact command exits `0`, reports `Status: warning-only`, records 2,671 warning findings and zero error findings, emits compile banners for `octet_lint_metadata`, `octet`, `octet_standards`, and `cargo_octet`, and reaches `Finished checking`.

Warning-only success follows the existing public exit-code contract. It is not a zero-finding claim and does not weaken lint levels or pass `--allow-style-violations`.

## Deterministic gate

`checks.x86_64-linux.cargo-octet-self-check` passed and retains three bounded cases:

1. The wrapped package checks a writable copy of the filtered source using the vendored Cargo configuration and a fresh target directory. The gate requires executable Clang and Mold paths, exact checked-in linker settings, all four workspace-library compile banners, the completed-build marker, a clean or warning-only summary, and zero error findings.
2. Malformed canonical metadata containing a non-string `default_scope` value exits `2` with the stable diagnostic before a Cargo sentinel can run.
3. Valid metadata with an invoked Cargo sentinel that exits non-zero produces an integration-failure summary and runner exit `2`.

The Rust regression test parses checked-in root `Cargo.toml` and requires exact scope and Cargo-tail arrays. Existing metadata precedence, missing metadata, malformed array, and Cargo compilation-failure tests remain intact.

## Checks

- Pre-change `config::tests`, missing-metadata integration, and malformed-metadata integration baselines — passed while the canonical repository command remained blocked as expected.
- Focused root-manifest, metadata, malformed-input, and Cargo-child failure tests — passed.
- `cargo fmt --all -- --check` — passed.
- Focused cargo-octet library/test Clippy with warnings and pedantic lints denied — passed.
- `nixfmt --check flake.nix` — passed.
- Exact `nix run .#cargo-octet -- check` — exit `0`, warning-only, 2,671 warnings, zero errors, all four workspace libraries observed.
- Dedicated `cargo-octet-self-check` Nix gate — passed.
- Nextest — 231 tests passed; 2 configured tests skipped.
- `nix flake check -L --option secret-key-files ''` — all evaluated `x86_64-linux` checks passed; incompatible systems were omitted.
- Native Cairn proposal, design, tasks, sync, strict validation, and typed policy freshness gates — passed.

## Traceability

Accepted spec `cairn/specs/runner-self-check/spec.md` is registered in both `octet-default` and `cairn-default` typed Nickel policy profiles. Every `octet.runner_self_check.*` requirement is linked from executable Rust evidence. Current policy hash is `b46b6eab0e9f52d31a96f37f271df81bc6d21820db44e4a40100c310bbaac125`.

The final traceability report preserves the inherited baseline exactly: 32 unrelated `octet.evidence_provenance.*` IDs remain missing and `octet.effect_closure.cli.check_failure` remains dangling. No self-check requirement is missing or dangling.

## Adversarial audit

Portfolio search rejected implicit loader defaults, top-level legacy metadata, and script-only scope overrides. An advisory audit raised false-completion risks around syntax-only validation, unobserved workspace scope, and build-time-only linker availability. The final gate closes those risks through actual packaged execution, exact compile-banner assertions for every selected workspace library, explicit runtime linker checks, and distinct pre-Cargo and post-Cargo negatives. Repository and Nix checks, not advisory prose, are the acceptance authority.

## Non-claims

This evidence covers the pinned packaged runner, toolchain, driver, lint library, root Cargo policy, and workspace library targets observed in this run. It does not cover tests, binaries, examples, benches, dependency behavior, macro-expansion completeness, runtime behavior, whole-program correctness, lint completeness, downstream configuration correctness, or release eligibility.

Development-shell and Nix evaluation continued after the known unavailable Git HTTPS helper and configured remote builder warnings, using pinned cached inputs and local builds.
