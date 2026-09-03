## Context

The current coverage rail accepts `cargo-llvm-cov-summary/v1`. It records line, branch, and function percentages plus caller-supplied regressions and threshold failures.

Run mode invokes `cargo llvm-cov --json --summary-only`, then parses combined process output as the normalized Octet schema.

The baseline file is hashed, but the pure core does not read its counts or calculate regressions.

## Decisions

### Decision: Harden the existing rail

**Choice:** Keep `coverage-baseline` and add a typed profile plus native adapter.

**Rationale:** Existing consumers retain a stable command while comparison authority moves into Octet.

### Decision: Pin collection semantics

**Choice:** Nickel profiles bind cargo-llvm-cov, Rust and LLVM toolchains, Nix closure, package and target scope, features, test selection, metrics, path policy, thresholds, limits, and non-claims.

The shell constructs fixed argv and rejects arbitrary command text or undeclared Cargo arguments.

**Rationale:** Coverage values are comparable only when collection semantics match.

### Decision: Normalize counts before percentages

**Choice:** The pure core extracts covered and total counts for each enabled metric from supported native machine output.

It computes display percentages through a declared precision and rounding policy. Decisions use integer counts or exact rational comparisons.

**Rationale:** Caller-provided floating percentages can drift or hide denominator changes.

### Decision: Compare current and baseline inside Octet

**Choice:** Octet reads, validates, and remeasures both reports. It computes regressions and threshold failures from normalized counts.

Comparability requires matching profile, toolchain, target, feature, test-selection, metric, path-policy, and source-scope identities.

**Rationale:** A baseline hash alone does not prove that regression fields came from that baseline.

### Decision: Use a dedicated machine artifact

**Choice:** cargo-llvm-cov writes supported machine output to a capability-rooted path. Stdout and stderr remain bounded diagnostic logs.

**Rationale:** Mixed process output is not a stable report contract.

### Decision: Preserve import compatibility

**Choice:** Existing normalized v1 imports remain accepted as bounded transcript evidence.

They cannot claim Octet-calculated baseline comparison unless both native parents are supplied and verified.

**Rationale:** Hardening new runs does not require breaking existing data exchange.

## Functional Core and Imperative Shell

The pure core validates profiles, native reports, normalized counts, path projections, comparability, thresholds, regressions, identities, and deterministic ordering.

The shell resolves pinned tools, creates capability-rooted output paths, executes fixed argv, reads bounded reports, and writes receipts atomically.

## Risks / Trade-offs

- Instrumentation can change timing or behavior. Receipts retain toolchain, target, and scope identity.
- Baselines can reward weak historical coverage. Profiles support absolute thresholds and regression policy as separate decisions.
- Source paths can leak host details. Normalization retains safe workspace-relative paths only.

## Validation

Positive fixtures cover stable counts, improved counts, explicit threshold success, and compatible legacy imports.

Negative fixtures cover regressions, denominator changes, incomparable scope, malformed output, unsafe paths, stale baseline, limits, tool failure, and overclaims.
