# Verification

Date: 2026-11-09

## Implemented boundary

- `crates/octet-merge-rail/` implements the bounded four-version relation over
  a hand-authored typed scalar IR with explicit assumptions, role-tagged
  identities, validated item mappings, dependency closure, deterministic
  outcomes, and identity-bound receipts.
- `tests/corpus.rs` executes the reviewed corpus and compares every case to an
  independently authored expectation table.
- `tests/negative_controls.rs` executes the required negative controls and
  additional malformed-inventory and domain-bound controls.
- `docs/semantic-merge-verification-rail.md` records the profile, relation,
  observed experiment results, go/no-go decision, and consumer non-claims.
- The rail is not wired into starter `cargo octet check`.

## Executed checks

All commands ran in this worktree with a dedicated target directory and
reduced build parallelism.

1. `CARGO_TARGET_DIR=/home/brittonr/.cache/octet-issue-1/target CARGO_BUILD_JOBS=2 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo check --workspace --all-targets`
   - Result: success, `Finished dev profile ... in 1m 15s`.
2. `CARGO_TARGET_DIR=/home/brittonr/.cache/octet-issue-1/target CARGO_BUILD_JOBS=2 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p octet-merge-rail`
   - Result: 9 corpus tests passed and 16 negative-control tests passed; 0 failures.
3. `cargo fmt --all -- --check`
   - Result: success, no diff.
4. `CARGO_TARGET_DIR=/home/brittonr/.cache/octet-issue-1/target CARGO_BUILD_JOBS=2 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo clippy -p octet-merge-rail --all-targets -- -D warnings`
   - Result: success, no warnings.
5. `CARGO_TARGET_DIR=/home/brittonr/.cache/octet-issue-1/target CARGO_BUILD_JOBS=2 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings`
   - Result: success, `Finished dev profile ... in 28.56s`.
6. `cargo run -p octet-merge-rail --bin octet-merge-rail-experiment`
   - Result: 6 cases, 6 expectation matches, precision 6/6, coverage 6/6,
     outcome counts `verified=3`, `counterexample=3`, `unknown=0`,
     `unsupported=0`, `timeout=0`, `tool-error=0`, and 80 deterministic work
     units.
7. `cairn validate --root .`
   - Result: `valid: true`, zero issues.
8. `cairn gate proposal add-bounded-semantic-merge-rail --root .`
   - Result: `PASS`, zero issues.
9. `cairn gate design add-bounded-semantic-merge-rail --root .`
   - Result: `PASS`, zero issues.
10. `cairn gate tasks add-bounded-semantic-merge-rail --root .`
    - Result: `PASS`, zero issues.

## Independent expectation comparison

| Case | Expected | Observed |
|---|---|---|
| identical | verified | verified |
| independent-changes | verified | verified |
| equivalent-edits | verified | verified |
| incompatible-changes | counterexample `incompatible@f` | counterexample `incompatible@f` |
| lost-changes | counterexample `lost-a-change@f` | counterexample `lost-a-change@f` |
| text-clean-semantic-interference | counterexample `interference@f` | counterexample `interference@f` |

These are observed counts for this corpus only. No performance, general
precision, universal-verification, or paper-derived claim is made.

## Negative controls exercised

Missing observable; vacuous precondition; changed callee; wrong source binding;
wrong subject binding; swapped roles; stale receipt; tampered receipt; unsupported construct; missing
call mapping; domain-bound overflow; interrupted evaluation budget; injected
backend failure; duplicate role; missing binding; missing role.

## Not run and open obligations

- `cargo octet check` and Nix flake checks were not run. This change
  intentionally does not add work to the starter check.
- No real Rust source-to-IR frontend or symbolic/external verifier was
  available. The typed IR is hand-authored, so Rust source-to-IR correspondence
  is unproven and production admission remains a no-go.
