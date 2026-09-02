## Context

Octet owns checked Rust rails and deterministic receipts. The behavioral model layer (Choregraph) will produce bounded model-check verdicts and witness traces, and Kamacite owns the interchange that maps foreign trace formats into stack identity. Octet needs its own admission rail that consumes those artifacts and rechecks a crate against them.

The stack pattern is to keep hard verification machinery external and keep identity, freshness, and receipts owned. Trace replay is deterministic work that belongs in Octet's checked surface.

## Decisions

### Decision: Spec-trace parity is a first-class Octet check

**Choice:** Add `spec-trace-parity` as a versioned Octet check that replays a pinned trace set against the target crate through an injected per-crate adapter.

**Rationale:** Parity between model and implementation is a checkable claim that gate owners must be able to admit or deny with receipts.

### Decision: Freshness binds every layer before replay

**Choice:** Before replay, require the model hash, checker identity and version, seed, and trace-set hash to match the declared pins. Any mismatch yields a deterministic denial before any trace step runs.

**Rationale:** Replaying stale traces can fabricate false parity. The rail must fail closed first.

### Decision: Replay is seed-bound and deterministic

**Choice:** Bind every replay run to an explicit seed and require identical inputs and seeds to produce identical step sequences and receipts.

**Rationale:** Reproducibility is the baseline for checked evidence.

### Decision: Parity receipts separate result classes

**Choice:** Emit receipts with parity, divergence, and unsupported-verifier classes, each with its own deterministic identity and non-claims.

**Rationale:** A divergence is different evidence from a missing adapter or an unknown trace format, and each must be recorded as its own claim.

### Decision: The adapter is injected, not ambient

**Choice:** Each target crate supplies its trace-replay adapter through the Octet adapter surface; Octet never guesses state mapping or action handling.

**Rationale:** Octet owns the rail and the receipt, while each crate owns the meaning of its own model projection.

## Functional core / imperative shell split

- **Pure core**: pin validation, freshness checks, trace-step sequencing against the declared adapter contract, verdict classification, and receipt construction and validation.
- **Imperative shell**: locate the crate, load adapter and traces, execute replay steps against the crate process, enforce time and step bounds, and persist receipts.

## Risks / Trade-offs

- Trace counts grow with model coverage. Enforce a bounded trace-set size and a step budget per run.
- Adapters can hide divergence by projecting state lossily. Require the adapter to declare a loss bound or state projection non-claim.
- ITF and engine formats drift. Consume traces only through a declared interchange identity and reject unknown formats.
- Replay against a live crate process is an effect; keep all decisions in the pure core and treat the process as an injected executor.

## Non-Goals

- No in-repo model checker or SMT engine.
- No claim that replayed parity covers all executions.
- No import of a foreign trace format as stack identity; format mapping belongs to Kamacite.
- No ambient authority for replay adapters.
