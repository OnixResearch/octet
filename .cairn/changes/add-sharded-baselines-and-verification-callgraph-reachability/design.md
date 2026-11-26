# Design: Per-Crate Sharded Baselines & Cross-Language Verification Reachability

## 1. Directory-Sharded Baselines & `#[allow]` Delta Ratchet (`cargo-octet/src/baseline.rs`)
- Accept a directory path for `--baseline` / `--write-baseline` reading/writing deterministic per-crate `octet-baseline/<crate>.json` shards (fully round-trip equivalent to monolithic JSON).
- Add strict ratchet mode enforcing `current_count <= baseline_count` per crate, rejecting any baseline entry for Tier-1 safety lints, and tracking total `#[allow(...)]` suppressions so net new lint escapes fail CI unless explicitly reviewed.

## 2. Verification Call-Graph Reachability & Assumption Inventory (`aeneas_execution.rs`, `charon_semantic/comparison.rs`)
- Combine Rust MIR/Charon call graphs with Aeneas-extracted Lean 4 theorem targets (Trailmark pattern) and an explicit `AssumedTrustBoundary` manifest (TxVerify pattern).
- Emit a structured reachability report flagging any untrusted entrypoint that reaches a guarded domain operation without traversing a proved kernel gate.

## Verification Plan
- `cargo test -p cargo-octet` verifying per-crate sharded round-trip equivalence, monotonic & `#[allow]` delta ratchet failures, and reachability failure when an untrusted entrypoint bypasses a proved kernel gate.
