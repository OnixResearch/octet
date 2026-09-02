# Proof Workspace State Specification Delta

## ADDED Requirements

### Requirement: Proof workspace state has a bounded contract

r[octet.proof_workspace_state.contract]

Octet MUST define versioned Rust-owned event and snapshot contracts for bounded proof groups, rails, targets, obligations, generations, sequences, states, diagnostics, receipt links, and non-claims.

#### Scenario: Complete contract data is admitted

- GIVEN events use supported schemas, closed states, current identities, safe text, and declared bounds
- WHEN contract validation runs
- THEN validation MUST admit the events for pure reduction
- AND equal inputs MUST retain equal canonical field ordering

#### Scenario: Contract data is malformed or excessive

- GIVEN data has an unknown state, duplicate node, missing parent, cycle, unsafe path, unbounded text, or exceeded node, depth, event, or artifact limit
- WHEN contract validation runs
- THEN validation MUST fail before snapshot publication

### Requirement: Proof state transitions are closed and deterministic

r[octet.proof_workspace_state.transitions]

The pure reducer MUST support only `stale`, `queued`, `running`, `blocked`, `passed`, `failed`, `cancelled`, and `unsupported` states. It MUST reject transitions outside the accepted transition table.

#### Scenario: Proof work completes

- GIVEN one current obligation moves from `queued` to `running` and then to `passed` with a matching receipt
- WHEN the reducer applies the ordered events
- THEN the current snapshot MUST report `passed`
- AND replaying the same events MUST produce byte-identical canonical state

#### Scenario: Terminal state changes without a new generation

- GIVEN an obligation is terminal and a later event attempts another terminal outcome in the same generation
- WHEN transition validation runs
- THEN validation MUST reject the duplicate terminal transition

### Requirement: Freshness follows exact generation identity

r[octet.proof_workspace_state.freshness]

Every workspace node MUST bind source, proof, policy, and verifier generation identity. A change to any bound identity MUST make prior terminal state stale before new work can become current.

#### Scenario: Proof input changes after a pass

- GIVEN a current obligation is `passed` and its source, proof, policy, or verifier identity changes
- WHEN the next generation is admitted
- THEN the prior result MUST be `stale`
- AND it MUST NOT satisfy current proof goal-state feedback

#### Scenario: Old event arrives after generation replacement

- GIVEN a new generation is current and an event from the prior generation arrives
- WHEN event admission runs
- THEN the event MUST be rejected as stale
- AND current state MUST remain unchanged

### Requirement: Event sequence is explicit

r[octet.proof_workspace_state.snapshot]

Events within one generation MUST use a monotone sequence. Snapshot construction MUST fail on sequence gaps, sequence regression, duplicate sequence values, parent cycles, or conflicting node identity.

#### Scenario: Ordered events form a snapshot

- GIVEN a complete contiguous event sequence for one valid proof tree
- WHEN the reducer builds the current snapshot
- THEN each node MUST have one deterministic current state and diagnostic order

#### Scenario: Event sequence is incomplete

- GIVEN an event sequence has a gap, regression, duplicate, or conflicting node update
- WHEN snapshot construction runs
- THEN construction MUST fail with a deterministic event diagnostic

### Requirement: Proof execution shell emits selected state artifacts

r[octet.proof_workspace_state.shell]

When an operator selects workspace-state output, the proof shell MUST emit bounded typed events during execution and MUST publish the current snapshot atomically under the selected artifact directory.

#### Scenario: Workspace-state output is selected

- GIVEN a valid proof plan and writable selected artifact directory
- WHEN `cargo octet prove --workspace-state` runs
- THEN the shell MUST write bounded event and current snapshot artifacts
- AND those artifacts MUST bind the proof plan and generation identity

#### Scenario: Workspace-state output is not selected

- GIVEN an operator runs starter checks, strict profiles, or ordinary proof commands without the option
- WHEN those commands complete
- THEN they MUST NOT require workspace-state artifacts or a live-state consumer

### Requirement: Workspace state has no proof authority

r[octet.proof_workspace_state.boundary]

Stale, queued, running, blocked, cancelled, and unsupported state MUST remain operator observation only. A passed state MAY link existing proof evidence only when the current generation and receipt identities match.

#### Scenario: Frontend displays current state

- GIVEN a frontend receives a valid current snapshot
- WHEN it renders the proof tree
- THEN it MAY present state, diagnostics, and receipt links
- AND it MUST NOT mutate state, execute proofs, assign evidence roles, or claim proof authority

#### Scenario: Nonterminal state is promoted

- GIVEN a report treats stale, queued, running, blocked, cancelled, or unsupported state as satisfied proof evidence
- WHEN claim-boundary validation runs
- THEN validation MUST fail

### Requirement: Dafny workspace code is not adopted

r[octet.proof_workspace_state.dafny_boundary]

This change MUST treat `dafny-lang/dafny` release `v4.11.0` at commit `fcb2042d6d043a2634f0854338c08feeaaaf4ae2` as a clean-room state-model reference. It MUST NOT add the Dafny language server, protocol, compiler, or runtime to Octet.

#### Scenario: Consumer requests Dafny protocol compatibility

- GIVEN a consumer requires Dafny language-server messages or undocumented workspace behavior
- WHEN Octet workspace-state compatibility is evaluated
- THEN the request MUST be unsupported until a separate Cairn change defines the contract

### Requirement: Proof workspace validation is adversarial

r[octet.proof_workspace_state.verification]

The implementation MUST include positive and negative fixtures for contracts, transitions, generations, sequences, trees, receipts, bounds, artifacts, default independence, presentation boundaries, and non-claims.

#### Scenario: State fixture matrix runs

- GIVEN valid pass, fail, block, cancel, unsupported, replacement, and replay fixtures plus malformed and stale fixtures
- WHEN focused workspace-state validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail at its declared boundary
