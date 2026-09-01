## Context

Octet needs two benchmark views. Performance benchmarks measure command cost and cache behavior. Signal benchmarks measure whether diagnostics match an adjudicated oracle.

General Rust microbenchmark frameworks do not solve both needs. End-to-end Octet commands also include compiler, Dylint, filesystem, cache, and artifact costs.

## Decisions

### Decision: Build one Octet-owned benchmark evidence rail

**Choice:** Extend `cargo octet evidence collect` with `benchmark-evidence` instead of adding several benchmark command families.

**Rationale:** Existing evidence collection already owns optional profiles, receipts, provenance, artifacts, and non-claims.

### Decision: Start without a benchmark framework dependency

**Choice:** Implement a small Rust harness that runs fixed profile commands and records raw samples.

Optional Criterion, hyperfine, or instruction-count adapters require later provider-specific changes.

**Rationale:** Octet first needs end-to-end and signal measurements. A microbenchmark framework alone cannot provide them.

### Decision: Use typed workload and command registries

**Choice:** Nickel profiles bind suite IDs, workload identities, command IDs, modes, caches, toolchains, environment requirements, sample plans, metrics, limits, and comparison policy.

The Rust runner maps command IDs to fixed argv. It rejects arbitrary command text and mutable unmeasured workload selection.

**Rationale:** Benchmark results have no meaning without exact workload and command identity.

### Decision: Separate cold, warm, and incremental modes

**Choice:** Profiles define cache preparation and source-change state for each mode. Results from different modes are never merged.

**Rationale:** Build and lint costs differ greatly between cold, warm, no-change, and incremental runs.

### Decision: Preserve raw samples and environment facts

**Choice:** Receipts bind every retained sample, command exit, wall duration, CPU duration, peak memory where supported, output size, cache state, and environment fact.

The harness records CPU, memory, kernel, toolchain, Nix closure, repository, load, governor, and isolation facts when they affect comparability.

**Rationale:** Summary values cannot explain noise or environment drift.

### Decision: Use robust comparison policy

**Choice:** The pure core computes declared summaries from retained samples. It preserves outliers and applies a named practical threshold plus uncertainty rule.

A comparison can report `improved`, `stable`, `regressed`, `inconclusive`, or `incomparable`.

**Rationale:** One noisy timing or a tiny numerical change must not become a release claim.

### Decision: Adjudicate signal independently

**Choice:** The signal oracle binds expected lint ID, location class, outcome, rationale, reviewer role, and source identity.

The core classifies true positives, false positives, false negatives, expected suppressions, unexpected suppressions, and inconclusive cases.

**Rationale:** Octet output cannot be its own correctness oracle.

### Decision: Keep public claims receipt-backed

**Choice:** A published metric must cite a verified receipt, profile, corpus, environment, sample range, comparison result, and interpretation boundary.

**Rationale:** Public performance or quality numbers without methodology would weaken Octet's evidence model.

## Functional Core and Imperative Shell

The pure core validates profiles, sample sets, environment comparability, signal oracles, finding alignment, summaries, thresholds, classifications, and deterministic ordering.

The shell prepares declared cache modes, runs fixed argv, captures bounded observations, reads findings, and writes artifacts atomically.

## Risks / Trade-offs

- Host timing remains noisy. The comparison can report `inconclusive` instead of forcing a result.
- Oracle review can be subjective. Receipts bind reviewer roles, rationales, and unresolved cases.
- Cross-project corpora can drift. Every workload uses an immutable revision or exact source snapshot identity.
- Benchmark execution can be expensive. Profiles bind sample, time, storage, and workload budgets.

## Validation

Positive fixtures cover stable, improved, regressed, and correctly classified signal cases.

Negative fixtures cover stale workloads, arbitrary commands, failed samples, invalid timing, environment drift, oracle conflicts, missing findings, limits, and overclaims.
