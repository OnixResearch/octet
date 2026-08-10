# Validation evidence

Date: 2026-08-09

## Scope

This change closes Octet validation failures. It does not increase lint limits, add warning-baseline exceptions, or claim whole-program correctness.

## Before the repair

- `checks.x86_64-linux.cargo-octet-self-check` exited without enough failure evidence.
- The first observed candidate had 1,857 identities: 1,800 accepted, 57 new, and 15 stale.
- `cargo test -p octet catalog_docs_match_inventory_counts_defaults_and_rows` failed on README total and namespace text.
- The accepted self-check at revision `9188039e` contained 1,815 identities and 2,665 finding occurrences.

## Diagnostic and source repair

- `flake.nix` now reports the failing self-check phase and bounded baseline counts, stale IDs, and new finding summaries.
- If structured artifacts are absent, the gate prints the human summary or at most 80 stderr lines.
- Nominal-domain metadata moved behind the public compatibility module into `identity/model.rs`, `identity/admission.rs`, `identity/evidence.rs`, and `identity/tests.rs`.
- Source repairs removed every unreviewed candidate warning without changing the existing limits.
- README and first-patch documentation now satisfy the catalog and positioning contracts.

## Baseline migration proof

The repository-owned migration checker is `evidence/migrate-self-check-baseline.rs` in this change package.

Its successful output MUST report:

```text
candidate=1813 rekeys=13 removals=2
```

Its accepted-run occurrence comparison MUST report:

```text
old_occurrences=2665 candidate_occurrences=2660 growth_groups=0 reduction_groups=5
```

The self-test covers these paths:

- positive: one exact normalized rekey and one reviewed removal are accepted;
- negative: an unpaired candidate identity is rejected;
- positive: lower occurrence multiplicity is accepted;
- negative: higher occurrence multiplicity is rejected.

The reviewed removals are:

```text
b3:5cda7b996474af06
b3:a7bc26bfc639e5fe
```

The 13 remaining candidate identities map one-to-one to stale predecessors by lint, group, severity, file, crate, and bounded normalized message. Only `excessive_file_length` source length and `module_file_count` count/path rendering are normalized.

Final baseline facts:

- baseline identity: `b3:cb7cb5c359de18a3b6f05014edb6bee45989f236e8404f3e7633b7d25e89f057`;
- baseline file BLAKE3: `c9ad8f73b84e0eb671e96e41dc8a2acb02e4131578d71df7ecfcb2c8d0d5f711`;
- migration checker BLAKE3: `6cf76d280970f5c885fa2fb410bae6c711a49e81547f6f9f3fd33d8031b266f3`;
- accepted identities: 1,813;
- accepted occurrences: 2,660;
- accepted current: 1,813;
- new: 0;
- stale: 0.

## Commands and results

Passed:

```text
nix build path:$PWD#checks.x86_64-linux.cargo-octet-self-check -L --no-link
CARGO_TARGET_DIR=$PWD/target/cargo-validation nix develop -c cargo test --workspace
CARGO_TARGET_DIR=$PWD/target/cargo-validation nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo fmt --all --check
nix build path:$PWD#checks.x86_64-linux.clippy -L --no-link
nix run path:$PWD#cairn -- validate --root .
nix run path:$PWD#cairn -- workflow status repair-validation-closure --root .
nix develop -c rustfmt --edition 2024 --check .cairn/changes/repair-validation-closure/evidence/migrate-self-check-baseline.rs
nix develop -c cargo -q -Zscript .cairn/changes/repair-validation-closure/evidence/migrate-self-check-baseline.rs --self-test
```

The isolated workspace test run passed all enabled tests. The five ignored tests retain their explicit opt-in contracts.

The first full flake run exposed `clippy::struct_excessive_bools` on the preserved public `WrapperFacts` compatibility API. A scoped compatibility annotation fixed that release-profile failure. The self-check remained at 2,660 occurrences and 1,813 accepted identities after the fix.

Final full check:

```text
nix flake check path:$PWD -L
```

Result: PASS. Nix reported `all checks passed!` across 112 checks. It omitted only the three incompatible non-host systems that the flake reports explicitly.

## Post-archive validation

Sync added both requirement IDs to `.cairn/specs/runner-self-check/spec.md`. The first traceability run then caught missing direct coverage markers for those accepted IDs.

The final source maps implementation to `flake.nix` and `cargo-octet/src/baseline.rs`. It maps verification to `docs/self-check-warning-budget.md` and the packaged gate.

Passed after sync and archive:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root $PWD --strict
nix build path:$PWD#checks.x86_64-linux.cairn-traceability -L --no-link
nix build path:$PWD#checks.x86_64-linux.cargo-octet-self-check path:$PWD#checks.x86_64-linux.clippy path:$PWD#checks.x86_64-linux.cairn-traceability -L --no-link
```

## Environment note

Nix repeatedly failed to connect to `ssh-ng://root@10.10.10.1`. Local fallback completed the recorded checks. The remote connection warning was not a source validation failure.
