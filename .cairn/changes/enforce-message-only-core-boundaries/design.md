# Design: Enforce message-only core boundaries

## Context

The Octet project architecture gate already evaluates roles, effects, ports, adapter placement, external-type leakage, resource handles, and effect closure. Its compiler Architecture IR records source and type facts under an explicit project policy.

The new mechanism must compose with that gate. A separate source scanner or naming convention would duplicate weaker logic and miss aliases or vendor types.

## Decisions

### Decision: Add a project-selected message-boundary mechanism

**Choice:** Extend the typed Nickel architecture policy with a `message_boundaries` mechanism.

Each declaration names a state-owner identifier, owner scope, state types, inbound and outbound message types, transition functions, effect-plan types, permitted shell or adapter scopes, configured runtime-handle families, and required evidence mode.

No policy selection means no message-boundary finding or compliance claim.

**Rationale:** The consuming project owns its state and protocol boundaries. Octet evaluates the declared contract rather than inventing one from paths.

### Decision: Use compiler type identity rather than suffix scanning

**Choice:** Resolve handle families through compiler type identities, configured provider identities, aliases, generic arguments, associated types, and public signature facts.

Built-in families cover standard sockets, Unix streams, Tokio and crossbeam senders or receivers, task handles, common future executors, and configured connection or client providers. Project policy can add exact provider type identities.

An unknown required type identity blocks `gate` and `admission`. It does not become a clean result.

**Rationale:** Names such as `Session` or `Client` are ambiguous. Exact compiler and provider identities reduce false positives and hidden escapes.

### Decision: Keep logical protocol state valid

**Choice:** Application-owned values such as `SessionId`, `StreamId`, `PeerRef`, `Generation`, `DeliveryState`, and finite stream phases remain valid when their fields contain no live runtime handle.

The evaluator rejects a logical wrapper only when compiler facts show a forbidden nested type or required facts remain unresolved.

**Rationale:** Deterministic cores must model protocol state. The rule forbids hidden runtime authority, not explicit state machines.

### Decision: Validate complete boundary paths

**Choice:** For each declared state owner, the evaluator checks these paths:

```text
inbound message type
  -> declared transition function
  -> explicit decision or state result
  -> outbound message or effect-plan type
```

The evaluator rejects undeclared public ingress, callback or closure parameters that target another owner, runtime handles in state or contracts, resource-handle return values, async or future transition returns when the policy requires synchronous core transitions, and infrastructure types in messages.

A project can declare multiple transition functions for one owner. Every production ingress must resolve to one declared boundary or an explicit local-pure classification.

**Rationale:** A clean message DTO does not help when another public method bypasses it.

### Decision: Compose shared-state checks without overclaiming

**Choice:** The message-boundary result consumes existing findings for globals, interior mutability, synchronization, effect closure, and role placement.

A core path through shared mutable state, callback capture, or runtime channel handle blocks the selected mechanism. Local mutation that does not escape remains valid.

Octet records the contributing finding identities instead of claiming complete alias or runtime reachability analysis.

**Rationale:** Message-only communication and exclusive state ownership depend on several existing bounded mechanisms.

### Decision: Keep adapters effectful and replaceable

**Choice:** Shell and adapter scopes can own live handles. Their application-facing contracts must accept or return admitted messages, events, decisions, or observations rather than runtime handles.

Only declared composition roots can select concrete adapters. The evaluator preserves existing dependency-direction and port-ownership rules.

**Rationale:** The goal is transport containment, not removal of stateful transport implementations.

### Decision: Preserve evidence roles

**Choice:** Add message-boundary policy identity, owner and boundary facts, type-coverage facts, findings, unknowns, contributing mechanism refs, waivers, status, and non-claims to the architecture receipt.

The receipt states that static conformance does not prove canonical wire encoding, live and simulated parity, complete scheduler control, runtime determinism, or release readiness.

**Rationale:** Molten and later consumers need stable static evidence without claim promotion.

## Functional Core and Imperative Shell

The Octet standards core validates policy values, normalizes type and boundary facts, evaluates paths, classifies findings, and constructs receipt values.

The cargo-octet shell loads policy, invokes Cargo and rustc, collects Architecture IR shards, reads generated projections, and writes evidence artifacts.

## Adversarial Audit

The gate must reject these cases:

- a type alias hides `TcpStream`;
- a `Vec<Option<Arc<Connection>>>` hides a vendor handle;
- a trait associated type resolves to a sender or receiver;
- an `impl Future<Output = Connection>` escapes a transition;
- a borrowed byte slice from a transport enters persistent core state;
- a callback closure captures another state owner or shared lock;
- `Arc<Mutex<State>>` bypasses the message boundary;
- a wrapper called `Message` contains an external SDK request;
- one public ingress bypasses the declared transition;
- inactive features contain the only compliant adapter;
- missing compiler shards or unresolved dynamic dispatch are treated as clean;
- a waiver binds stale source or policy bytes.

The gate must accept these cases:

- a core stores logical session identifiers and finite phases;
- a pure helper call remains inside one state owner;
- an adapter owns an Iroh connection and emits an owned canonical application message;
- an application shell owns a bounded mailbox executor without exposing its handle to the core.

## Validation

Positive and negative fixtures must include direct, aliased, generic, associated-type, async, callback, shared-state, feature, and provider-configured cases.

Repeated runs under unchanged source, policy, target, features, and toolchain must produce byte-identical normalized facts and receipt identities.
