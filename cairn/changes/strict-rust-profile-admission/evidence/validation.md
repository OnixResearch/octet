# Strict Rust profile admission validation

Validation date: 2026-07-11

## Pre-change baseline

The following checks passed before core implementation:

| Command | Outcome |
| --- | --- |
| `nix run path:.#cairn -- validate --root .` | PASS; 65 accepted specs and the six pre-existing active changes validated. |
| `nix develop -c cargo test -p octet-lint-metadata` | PASS; the crate had no pre-existing unit tests. |
| `nix develop -c cargo test -p cargo-octet --lib` | PASS; 287 tests. |
| `nix develop -c cargo test --lib config::tests` | PASS; 21 focused lint-config tests. |

## Focused post-change verification

| Command | Outcome |
| --- | --- |
| `nix develop -c cargo fmt --check` | PASS. |
| `nix develop -c cargo test -p octet-lint-metadata` | PASS, including positive catalog resolution and negative duplicate/unknown/scope/non-claim cases. |
| `nix develop -c cargo test -p cargo-octet --lib` | PASS, including CLI selection, workspace-scope preparation, function-address floors, BLAKE3 identity, embedded-source coverage, and default-hash compatibility. |
| `nix develop -c cargo test --lib config::tests` | PASS; 23 tests, including required-lint strengthening and missing-core rejection. |
| `nix develop -c cargo test --test consumer_integration profile -- --nocapture` | PASS; known/unknown profile handling, CLI precedence over ambient profile state, environment scrubbing, missing-core rejection, and profile-bound artifact verification. |
| `nix develop -c cargo test --test consumer_integration deterministic_core_uses_reviewed_core_scope_function_address_baseline -- --exact --nocapture` | PASS; observe bootstrap followed by deny-mode `deterministic-core` admission over the configured core scope. |
| `nix develop -c cargo test --test consumer_integration` | PASS; all 77 runner integration tests, including installed-runner embedded-source cache paths. |
| `nix build .#checks.x86_64-linux.strict-profile-catalog --no-link -L` | PASS; Nickel typecheck, valid fixtures, invalid fixtures, and byte-for-byte generated JSON freshness. |
| `nix run .#octet-policy-validate` | PASS; the new Nickel-authored config family is registry-valid. |
| `nix build .#cargo-octet --no-link -L` | PASS; Nix-packaged canonical and compatibility binaries built with the expanded embedded-source payload. |
| `nix develop -c cargo clippy -p cargo-octet --lib --bins -- -D warnings -A dead-code` | PASS for the changed runner production targets; `dead_code` is allowed here only to isolate the unchanged Trellis intake baseline described below. |
| `nix develop -c cargo clippy -p octet-lint-metadata --all-targets -- -D warnings` | PASS. |
| `nix develop -c cargo clippy -p octet --lib -- -D warnings` | PASS. |
| `git diff --check` | PASS. |

## Real wrapped-driver smoke

A temporary dependency-free Rust workspace used this source:

```rust
pub fn add_checked(left: u32, right: u32) -> Option<u32> {
    left.checked_add(right)
}
```

Its workspace metadata selected the package with `cargo_check_args = ["--lib"]`, and its `dylint.toml` contained canonical `[octet]` configuration. The exact commands were:

```bash
nix run path:/home/brittonr/git/OnixResearch/octet#cargo-octet -- \
  check --profile robust --function-address-path src --artifact-dir artifacts
nix run path:/home/brittonr/git/OnixResearch/octet#cargo-octet -- \
  artifact verify --artifact-dir artifacts --output-format json
```

Observed result:

- Cargo and the real wrapped Dylint driver succeeded.
- `summary.txt` reported `Profile: robust`, status `warning-only`, one expected observe-mode `function_address_missing` advisory, zero errors, and exit code zero.
- `status.json` recorded `profile_name = "robust"`, `config_hash = "b3:3fe159f078bba5dc6df670101f9ee05737b0e51f6461f53d71d1e93594aa3fb6"`, and `profile_hash = "b3:dfc179848d67f77a14e29f3ad66be9d1b0130630b53c261ae7208d01e1b30038"` under the pinned nightly toolchain.
- `function-addresses.json` reported mode `observe`, one `src/lib.rs::add_checked` record, source-token-set dependency identity, and the existing source-shape non-claims.
- Artifact verification returned status `valid`, checked scope `profile:robust`, and an empty diagnostics array.

The observe advisory is expected bootstrap debt, not a clean function-address admission claim.

## Broad Clippy baseline blocker

The unfiltered command

```bash
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
```

remains blocked by diagnostics in unchanged pre-existing files:

- dead code in `cargo-octet/src/imported_proof.rs`;
- one test-only dead constant in `cargo-octet/src/valence_cutover.rs`;
- one test-only collapsible `if` in `cargo-octet/src/source_acquisition_receipts.rs`.

The strict-profile implementation initially exposed one additional `derivable_impls` diagnostic after adding ordering to `FunctionAddressMode`; that diagnostic was fixed by deriving `Default`. A final broad rerun showed only the unchanged files above. The focused production-target Clippy commands pass, so this change does not add a remaining Clippy diagnostic. Full `nix flake check` was not claimed because it includes the same pre-existing all-targets Clippy blocker.

## Cairn gates

After implementation and focused validation, these commands all passed:

```bash
nix run path:.#cairn -- validate --root .
nix run path:.#cairn -- gate proposal strict-rust-profile-admission --root .
nix run path:.#cairn -- gate design strict-rust-profile-admission --root .
nix run path:.#cairn -- gate tasks strict-rust-profile-admission --root .
```

The task gate returned `valid: true`, `verdict: PASS`, and no issues. This evidence records source-shape policy and artifact-integrity checks only; it does not promote the change to transitive effect proof, compiler correctness, whole-program correctness, verifier soundness, or release eligibility.
