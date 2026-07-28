## Context

Effect closure v1 intentionally denies unsupported source shapes. Charon supplies selected semantic facts through a separate recorded-only rail.

MIRAI analyzes Rust MIR and can report panic, contract, taint, and call-graph observations. Its rustc coupling and diagnostic stability require measurement before adoption.

## Decisions

### Decision: Run a pilot before adding a product rail

**Choice:** Build a selected-corpus provider package and decision receipt without registering MIRAI as a general evidence rail.

**Rationale:** The pilot must prove added value and maintenance feasibility before Octet accepts another semantic provider.

### Decision: Pin every provider input

**Choice:** Bind exact MIRAI revision, rustc toolchain, target, features, command argv, environment allowlist, Nix derivation, closure graph, and source identities.

**Rationale:** MIRAI depends on compiler internals. Ambient tool selection cannot support repeatable evidence.

### Decision: Start with comparable properties

**Choice:** Select small fixtures for panic reachability, checked preconditions, call edges, and effect-relevant operations.

Taint or constant-time fixtures enter the pilot only after a stable output and oracle contract exists.

**Rationale:** Comparable properties create a meaningful baseline against Charon and source-token evidence.

### Decision: Permit a blocked pilot outcome

**Choice:** If the pinned provider cannot build, emit stable machine-readable output, or analyze the corpus within limits, record an exact blocked result.

**Rationale:** A truthful blocker is better than a parser that depends on unstable human diagnostics.

### Decision: Preserve independent authority

**Choice:** Compare each aligned fact as `agree`, `disagree`, `mirai-only`, `charon-only`, `source-token-only`, or `unknown`.

Every row retains parent identities. Agreement does not promote evidence or prove semantic equivalence.

### Decision: Require a measured promotion gate

**Choice:** The final decision uses declared thresholds for supported fixture rate, deterministic replay, diagnostic stability, unique useful findings, false positives, runtime, and maintenance cost.

Thresholds live in typed Nickel policy. A decision record names the observed measurements and failed criteria.

**Rationale:** Tool adoption must follow evidence instead of feature claims.

## Functional Core and Imperative Shell

The pure core validates provider observations, normalizes supported facts, aligns parent identities, classifies comparisons, and evaluates the decision policy.

The shell builds or selects the pinned provider, runs fixed argv with bounded resources, reads artifacts, and writes receipts.

## Risks / Trade-offs

- MIRAI output or compiler compatibility can drift. Exact pins and a blocked outcome contain this risk.
- A small corpus can overstate usefulness. The decision receipt states corpus size, scope, and residual uncertainty.
- Cross-tool agreement can create false confidence. Comparison rows remain recorded observations with explicit non-claims.

## Validation

Positive fixtures include a supported safe function, a reachable panic, and a declared precondition result.

Negative fixtures cover unsupported syntax, malformed output, stale source, toolchain drift, timeout, disagreement, missing parents, and claim promotion.
