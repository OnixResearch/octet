# Tasks

## Phase 1 — Per-Crate Sharded Baselines & `#[allow]` Ratchet
- [ ] Add directory-sharded `octet-baseline/<crate>.json` support and strict per-crate monotonic + Tier-1-zero-baseline enforcement in `cargo-octet/src/baseline.rs`.
- [ ] Add explicit `#[allow(...)]` suppression delta ratchet reporting and fail-closed gate.

## Phase 2 — Cross-Language Reachability & Verification
- [ ] Implement untrusted-entrypoint -> proved-Lean-theorem reachability and `AssumedTrustBoundary` inventory reporting in `cargo-octet/src/{aeneas_execution.rs, charon_semantic/comparison.rs}`.
- [ ] Add unit and integration tests in `cargo-octet` covering sharding, ratchets, and unproved bypass detection.
