# Why

Cross-vault findings from `x-bookmarks-mneme` (`bookmarks/2026-11-25-2125561387241857029.adoc` *Trailmark v0.9.0* extending call graphs to Lean 4/Dafny/TLA+ to audit untrusted-input reachability; `bookmarks/2026-11-24-2125303008183333092.adoc` *TxVerify* explicit core/assumption splits) and `onix-insights-mneme` (`13-mneme-post-merge-audit-invariants.adoc`, `16-ecosystem-swarm-invariants-and-lessons-2026-11.adoc` `octet#46` monotonic warning ratchet + zero new `#[allow]` escapes) highlight two high-leverage upgrades for `cargo-octet`:

1. Monolithic `octet-baseline.json` files cause multi-MB merge conflicts across every parallel swarm wave, and need native per-crate directory sharding (`octet-baseline/<crate>.json`) with monotonic `new <= base` and `#[allow]` delta ratchets.
2. Current Aeneas/Charon audits check syntactic `pub fn` inventory presence rather than cross-language call-graph reachability from untrusted entrypoints to proved Lean 4 theorems plus explicit `AssumedTrustBoundary` inventories.

# What Changes

- Extend `cargo-octet/src/baseline.rs` (`write_baseline`, `evaluate_baseline`, `BaselineEvaluation`) to support directory-sharded `octet-baseline/<crate>.json`, enforce strict per-crate monotonic ratchets (`new <= base`, zero baseline for Tier-1 safety lints), and add an explicit `#[allow]` suppression delta ratchet.
- Extend `cargo-octet/src/aeneas_execution.rs` and `cargo-octet/src/charon_semantic/comparison.rs` with a cross-boundary reachability and `AssumedTrustBoundary` report mapping untrusted Rust entrypoints to proved Lean 4 theorems (failing closed on unproved bypasses).
