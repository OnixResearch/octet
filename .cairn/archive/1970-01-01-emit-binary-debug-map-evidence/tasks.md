# Tasks: Emit binary debug-map evidence

## Contracts and dependencies

- [x] [serial] Define the Nickel-authored `elf-dwarf-v1` profile, explicit source-root mappings, and bounded resource limits r[octet.evidence_provenance.binary_debug_map_profile]
- [x] [serial] Add pinned `object`, `gimli`, and `rustc-demangle` dependencies to a std evidence producer with license and source review r[octet.evidence_provenance.binary_debug_map_profile]
- [x] [serial] Define Rust-owned receipt DTOs and generate their Nickel compatibility contract r[octet.evidence_provenance.binary_debug_map_profile]

## Core and shell

- [x] [serial] Implement pure bounded object, DWARF, symbol, range, line, and source-map normalization over supplied bytes r[octet.evidence_provenance.binary_debug_map_linkage]
- [x] [serial] Implement exact binary, debug companion, function address, and typed requirement linkage r[octet.evidence_provenance.binary_debug_map_linkage]
- [x] [serial] Add the explicit `binary-debug-map` collection shell with capability-rooted reads and atomic bundle publication r[octet.evidence_provenance.binary_debug_map_handoff]

## Verification

- [x] [parallel] Add positive inline-DWARF and split-debug fixtures with deterministic receipt goldens r[octet.evidence_provenance.binary_debug_map_linkage]
- [x] [parallel] Add negative fixtures for malformed sections, wrong companions, unmapped paths, stale spans, ambiguity, truncation, and every resource-limit class r[octet.evidence_provenance.binary_debug_map_failure_boundary]
- [x] [parallel] Add Valence handoff fixtures that preserve recorded-only roles and reject claim promotion r[octet.evidence_provenance.binary_debug_map_handoff]

## Documentation and closeout

- [x] [parallel] Document supported formats, profile limits, stripped-binary behavior, privacy, and non-claims r[octet.evidence_provenance.binary_debug_map_failure_boundary]
- [x] [serial] Run focused tests, generated-contract freshness, artifact verification, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.evidence_provenance.binary_debug_map_profile]
- [x] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.evidence_provenance.binary_debug_map_handoff]

## Evidence

Focused checks, run from the change worktree with
`CARGO_TARGET_DIR=/home/brittonr/.cache/octet-work-emit-binary-debug-map-evidence/target`,
`CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_PROFILE_DEV_DEBUG=0`, and
`CARGO_INCREMENTAL=0`:

- `cargo check --workspace --all-targets`: passed.
- `cargo test -p cargo-octet --lib binary_debug_map`: 12 passed, covering the
  admitted profile, inline and split debug fixtures, deterministic repeat
  receipts, wrong companions, malformed and unsupported inputs, unmapped paths
  with and without the exclude policy, stale spans, ambiguous candidates, every
  resource-limit class, shell byte limits, recorded-only handoff, and bundle
  verification.
- `cargo test -p cargo-octet --lib evidence_contracts`: 8 passed, including
  generated-contract freshness and valid/invalid Nickel contract fixtures.
- `cargo test -p cargo-octet --lib evidence_rails`: 77 passed.
- `cargo test -p cargo-octet --lib artifact`: 129 passed.
- `cargo test -p cargo-octet --lib cli`: 54 passed.
- `cargo test -p cargo-octet --lib`: 802 passed, 26 ignored.
- `cargo fmt --check`: clean.
- `cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings`: clean.
- `nickel typecheck standards/binary-debug-map/profiles.ncl` and
  `nickel export --format json standards/binary-debug-map/profiles.ncl` match
  the checked `standards/binary-debug-map/profiles.json`.
- `cairn validate --root .`: no issues.
- `cairn gate proposal|design|tasks emit-binary-debug-map-evidence --root .`:
  all PASS.

Dependency review: `object = "=0.37.3"`, `gimli = "=0.34.0"`, and
`rustc-demangle = "=0.1.28"` are pinned exactly, checksum-bound in `Cargo.lock`,
and dual-licensed Apache-2.0 OR MIT. The producer uses only read features; test
fixtures add the write features in dev-dependencies.

No task is blocked. The rail is deliberately absent from starter
`cargo octet check`, and no consumer or publication authority is claimed.
