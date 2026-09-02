# Design: Proof workspace state

## Context

Proof goal-state feedback explains evidence obligations from completed Octet facts. A live workspace view needs a separate state machine for work that has not produced current evidence.

## Decisions

### Decision: Use a pure event reducer

**Choice:** A pure reducer consumes ordered events and returns the current proof tree plus deterministic diagnostics.

**Rationale:** Frontends can replay and test state without process, filesystem, clock, or terminal access.

### Decision: Bind every node to one generation

**Choice:** Each group, rail, target, and obligation names source, proof, and policy identities through one generation record.

**Rationale:** Display labels and paths are not strong enough to distinguish current proof work from stale work.

### Decision: Use a closed state set

**Choice:** V1 supports `stale`, `queued`, `running`, `blocked`, `passed`, `failed`, `cancelled`, and `unsupported`.

**Rationale:** Unknown states cannot silently appear as success.

### Decision: Make freshness a transition rule

**Choice:** A new source, proof, policy, or verifier generation moves all prior terminal results to `stale` before new work can become current.

**Rationale:** A previously passing receipt must not remain current after its proof inputs change.

### Decision: Keep event order explicit

**Choice:** Each generation uses a monotone sequence. Gaps, regressions, duplicate terminal events, and cross-generation updates fail closed.

**Rationale:** File arrival order or frontend timing cannot define canonical state.

### Decision: Emit artifacts only when selected

**Choice:** `cargo octet prove --workspace-state` writes a bounded newline-delimited event artifact and an atomic current snapshot under the selected artifact directory.

**Rationale:** Starter and ordinary proof paths remain stable.

### Decision: Presentation has no proof authority

**Choice:** Frontends can map state to text, symbols, color, and navigation. They cannot change state or evidence roles.

**Rationale:** Octet remains the owner of proof status and receipt linkage.

## Functional Core and Imperative Shell

The pure core validates nodes, generations, transitions, event ordering, freshness, receipt linkage, bounds, and snapshots.

The shell observes proof execution, creates typed events, writes bounded event data, and publishes snapshots atomically.

## Data Flow

```text
proof plan and source generation
  -> typed proof events
  -> pure ordered reducer
  -> current workspace snapshot
  -> optional terminal frontend
```

## Risks and Controls

- Interrupted runs can leave no terminal event. The snapshot remains `running` or becomes `cancelled` through an explicit later event.
- Large proof trees can exhaust memory. Policy binds node, depth, event, text, and artifact limits.
- Frontends can hide stale state. Accessibility text and fixture rules require visible state labels.

## Validation

Positive fixtures cover complete pass, expected failure, blocked work, cancellation, generation replacement, and deterministic replay.

Negative fixtures cover sequence gaps, sequence regression, duplicate terminal events, missing parents, cycles, stale receipt promotion, unknown states, exceeded bounds, and overclaims.
