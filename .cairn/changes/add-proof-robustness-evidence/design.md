# Design: Proof robustness evidence

## Context

Octet already binds a proof to exact source, verifier, solver, toolchain, command, assumptions, and artifact identities. This change adds evidence about repeated proof behavior and visible proof inputs.

## Decisions

### Decision: Extend existing proof rails

**Choice:** Add an optional robustness profile to `cargo octet prove` and reuse the existing proof planner, bounded command shell, receipt validation, and artifact verifier.

**Rationale:** A second proof runner would duplicate authority and weaken receipt linkage.

### Decision: Keep the profile typed and explicit

**Choice:** A Nickel map owns selected rail IDs, ordered unique seeds, provider metric names, run limits, classification policy, required audit facts, and non-claims.

**Rationale:** Ambient seeds, hidden thresholds, and implicit retry behavior cannot support reviewable evidence.

### Decision: Compare provider resource units, not wall time

**Choice:** The core compares only declared provider resource metrics. Wall duration remains a diagnostic observation.

**Rationale:** Host scheduling noise must not become proof-brittleness evidence.

### Decision: Separate declared and observed trust facts

**Choice:** The receipt keeps policy assumptions, accepted trust boundaries, source-scan facts, no-cheating mode, and provider-reported axiom use as separate fields.

**Rationale:** A missing observed use does not prove that an assumption is irrelevant. A declared boundary does not prove that the solver used it.

### Decision: State dependency coverage exactly

**Choice:** Each provider reports `complete`, `axiom-usage-only`, or `unsupported` dependency coverage. Verus starts as `axiom-usage-only` unless stronger extraction is proven.

**Rationale:** Dafny proof-dependency analysis cannot be assumed for another verifier.

### Decision: Robustness evidence never promotes proof role

**Choice:** A robustness receipt supplements matching child proof receipts. It remains recorded evidence with explicit boundary facts.

**Rationale:** Seed stability and resource stability do not prove the theorem or verifier soundness.

### Decision: Defer direct Dafny execution

**Choice:** Record the pinned Dafny source as a design reference only.

**Rationale:** Dafny execution would add .NET, Boogie, Z3, compiler, and backend trust surfaces. It requires a separate adoption decision.

## Functional Core and Imperative Shell

The pure core validates profiles, run observations, identity linkage, audit facts, resource summaries, classifications, and receipt ordering.

The shell selects pinned proof commands, injects declared seeds, enforces limits, captures bounded outputs, reads source facts, and writes artifacts.

The core never reads files, spawns processes, selects ambient tools, reads clocks, or emits terminal output.

## Data Flow

```text
Nickel robustness profile
  -> proof-run plan
  -> bounded child proof receipts
  -> typed run observations
  -> pure robustness classification
  -> proof-robustness receipt
  -> artifact verification and optional Valence export
```

## Risks and Controls

- Provider metrics can change. Exact provider and metric identities make incompatible observations fail closed.
- Seed changes can expose solver search differences. The result remains robustness evidence, not solver soundness evidence.
- Source scans can miss semantic assumptions. The receipt states scan coverage and preserves unknown classes.
- Large matrices can waste resources. Profiles bind run, time, output, and retained-artifact limits.

## Validation

Positive fixtures cover stable results, declared trusted boundaries, exact axiom-use facts, and deterministic receipt identity.

Negative fixtures cover changed outcomes, missing runs, resource-policy failures, stale child receipts, changed assumptions, unsupported coverage promotion, malformed metrics, timeout, and overclaims.
