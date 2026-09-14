# Membership foundation evidence

## Historical phase-one result

This record covers only the first implementation phase.
The [role-binding record](role-binding-evidence.md) covers the next phase. The provider change remains active and unpublished.
Noble still pins `74a5d6b057272b149ae86049107cba5eac986b28`.
Neither repository claims that integration-test roles now pass the gate.

| Check | Result |
|---|---|
| Pre-change Architecture IR baseline | 23 tests passed |
| Pre-change project gate baseline | 27 tests passed |
| Pre-change artifact baseline | 37 tests passed |
| Shared-membership control before implementation | Failed: `normalized IR must retain unit membership` |
| Updated Architecture IR tests | 27 passed, including contract freshness |
| Standards package | 174 unit tests and 24 integration tests passed |
| Artifact module | 37 tests passed |
| Strict Clippy across the three affected packages | Passed with all targets, all features, and `-D warnings` |
| Nix runner build and formatting | Passed |
| Nix architecture-policy check | Passed, including generated Nickel contract typechecking |
| Actual Noble compiler run | All six selected units retained memberships in IR v2 |
| Candidate artifact verification | Valid bundle, with architecture still `integration-failure` |

The 262 targeted tests do not represent the full provider test suite.
The project gate tests are part of the 174 standards unit tests, not additional tests.
No lint, target, source-role declaration, or production requirement changed.

## Evidence boundary

The normalizer records the fact and unknown sets from each validated shard before cross-unit deduplication.
The new IR identity includes the complete membership relation.
Eleven malformed-relation cases and six ordering or empty-subject cases reject after the enclosing identity changes.
A missing shared edge changes the identity but can leave a structurally possible relation.
The corresponding control explicitly preserves that limitation: consistency is not authenticity.

The actual Noble run produced IR v2 and complete coverage for six compilation units.
The two `source-role-unit-mismatch` findings remain. A valid failure receipt does not establish architecture acceptance.
The gate still needs exact target-root role binding, a typed production view, and explicit test-only dispositions.
Test-only witnesses, ports, no-std facts, and unknowns must not satisfy production obligations.
Those controls remain open. No source-role exception is enabled in this phase.

## Reproduction

Run from `/home/brittonr/git/octet-test-ownership`.
The unchanged published provider worktree supplies the pinned development environment, not candidate implementation code.
Cargo compiles candidate sources from the current directory.
The same locked Nix vendor configuration supplies external dependencies.

```sh
export TMPDIR=/home/brittonr/git/noble/.pi/m1-test-ownership/tmp
export CARGO_HOME=/home/brittonr/git/noble/.pi/m1-octet-repair/cargo
export CARGO_TARGET_DIR=/home/brittonr/git/noble/.pi/m1-test-ownership/target
unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
nix --option min-free 0 develop /home/brittonr/git/octet-unprotected-receipts -c sh -c '
  cargo test -p octet-architecture-ir --all-targets --locked --offline &&
  cargo test -p octet-standards --all-targets --all-features --locked --offline &&
  cargo test -p cargo-octet --lib artifact::tests --locked --offline &&
  cargo clippy -p octet-architecture-ir -p octet-standards -p cargo-octet --all-targets --all-features --locked --offline -- -D warnings
'
```

`min-free = 0` applies only to these invocations. It avoids an automatic-GC lock wait after disk space became available.
No global Nix configuration changed. `/tmp` remains full, so commands use the private temporary directory.
An earlier shared-target run could not find the new IR export while compiling `cargo-octet`.
The fresh-target retry passed without source changes. The failed run remains available for review.

## Retention and next phase

Persistent evidence is in `/home/brittonr/git/noble/.pi/m1-test-ownership/`.
It includes command receipts, baseline and failed runs, source snapshots, current test logs, compiler observations, and artifact verification.
Key tasks: 1966 (baseline), 1972 (red control), 1980 (first green), 1995 (artifact baseline), 2002 (isolated quality).
Nix tasks were 2004 and 2007. Actual compiler collection was 2009. Artifact verification was 2011 and 2014.

The two bounded implementation rounds completed membership construction and its adversarial checks.
The next phase binds independent integration-test roots before the gate uses that membership to interpret roles.
No sync, archive, commit, publication, Noble repin, extraction, or M1 acceptance occurred in this phase.
