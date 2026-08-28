# Message Boundary Architecture Specification Delta

## ADDED Requirements

### Requirement: Message-boundary policy is explicit and typed

r[octet.message_boundary.policy]

An active project architecture policy MAY select a message-boundary mechanism with declared state owners, owner scopes, state types, inbound and outbound message types, transition functions, effect-plan types, permitted runtime-handle scopes, provider type identities, bounds, evidence mode, and non-claims.

Octet MUST NOT infer message-boundary compliance when the mechanism is absent.

#### Scenario: Complete policy activates the mechanism

- GIVEN a project selects the message-boundary mechanism with complete bounded declarations
- WHEN project architecture policy validation runs
- THEN Octet MUST accept the declarations for compiler-backed evaluation
- AND it MUST bind the policy and deterministic export identities.

#### Scenario: Boundary declaration is incomplete

- GIVEN a selected mechanism omits an owner, scope, message type, transition, bound, or required non-claim
- WHEN policy validation runs
- THEN validation MUST fail with a deterministic field diagnostic.

### Requirement: Compiler facts cover boundary-relevant types

r[octet.message_boundary.compiler_facts]

Octet MUST collect bounded compiler facts for declared state fields, public and selected private signatures, aliases, generic arguments, associated types, trait objects, lifetimes, async or future returns, callback parameters, constructors, conversions, feature state, targets, and configured provider type identities.

#### Scenario: Alias resolves to a runtime handle

- GIVEN a core state field uses an alias that resolves to a live connection type
- WHEN Architecture IR normalization runs
- THEN the normalized facts MUST retain the resolved type identity and alias path.

#### Scenario: Required type cannot be resolved

- GIVEN a required boundary type remains unresolved after compiler collection
- WHEN architecture evaluation starts
- THEN the result MUST retain an unknown fact
- AND strict modes MUST NOT treat the boundary as clean.

### Requirement: Every declared owner boundary uses admitted messages

r[octet.message_boundary.path]

For each declared state owner, Octet MUST evaluate each production ingress through an admitted inbound message, declared transition, explicit state or decision result, and admitted outbound message or effect-plan path.

Undeclared ingress, cross-owner callbacks, resource-handle returns, and bypass paths MUST block `gate` and `admission` modes.

#### Scenario: Complete transition path passes

- GIVEN a state owner receives an application-owned message and invokes a declared pure transition
- AND the transition returns explicit state, outbound messages, and effect plans
- WHEN message-boundary evaluation runs
- THEN the path MUST satisfy the declared boundary mechanism.

#### Scenario: Public method bypasses the message transition

- GIVEN a core exposes another production ingress that can mutate the state owner without an admitted message
- WHEN message-boundary evaluation runs
- THEN the architecture result MUST block and identify the bypass path.

### Requirement: Runtime handles do not escape into cores

r[octet.message_boundary.handle_containment]

Declared core state, message, port, transition, decision, event, and effect-plan types MUST NOT contain live socket, connection, endpoint, client, sender, receiver, executor, task, borrowed transport buffer, or configured vendor-handle types.

The rule MUST inspect aliases, nested generic arguments, associated types, trait objects, return values, and configured provider identities.

#### Scenario: Adapter contains a connection

- GIVEN a declared outbound adapter owns a live Iroh connection
- AND its application-facing port accepts and returns only admitted application values
- WHEN architecture evaluation runs
- THEN the connection MUST be accepted in the adapter scope
- AND it MUST NOT appear in core contract facts.

#### Scenario: Nested generic hides a connection

- GIVEN a core message contains `Option<Arc<Connection>>` through aliases or re-exports
- WHEN architecture evaluation runs
- THEN the result MUST block and report the resolved connection type path.

#### Scenario: Logical session value is handle-free

- GIVEN a core stores an application-owned session identifier, generation, and finite phase
- AND compiler facts show no runtime handle field
- WHEN architecture evaluation runs
- THEN handle containment MUST accept the logical protocol state.

### Requirement: Shared mutable paths cannot bypass messages

r[octet.message_boundary.shared_state]

A selected message-boundary mechanism MUST compose the existing global-state, interior-mutability, synchronization, capability, role, port, dependency, and effect-closure results for its declared core scopes.

A cross-owner shared mutable path or callback capture MUST block. Local mutation that cannot escape one state owner MAY remain valid.

#### Scenario: Shared lock crosses owners

- GIVEN two declared state owners mutate one `Arc<Mutex<State>>`
- WHEN message-boundary evaluation consumes compiler and existing mechanism facts
- THEN the result MUST block and link the contributing shared-state findings.

#### Scenario: Local mutation stays private

- GIVEN one transition uses local mutation that cannot escape its call
- WHEN message-boundary evaluation runs
- THEN the mechanism MUST NOT reject the transition solely because local mutation occurred.

### Requirement: Unknown required facts fail closed

r[octet.message_boundary.unknowns]

In `gate` and `admission` modes, missing compiler shards, unresolved required types, unresolved dynamic dispatch, omitted targets, omitted features, stale provider identities, and incomplete production ingress coverage MUST block message-boundary admission.

Inventory and advisory modes MUST retain these unknowns without reporting a clean boundary.

#### Scenario: Dynamic callback target is unresolved

- GIVEN a production callback can reach another state owner but its target remains unresolved
- WHEN strict message-boundary evaluation runs
- THEN evaluation MUST block with the exact unknown target fact.

#### Scenario: Advisory mode retains uncertainty

- GIVEN the same unresolved callback is evaluated in advisory mode
- WHEN the result is rendered
- THEN the unknown fact MUST remain visible
- AND the result MUST NOT claim message-boundary conformance.

### Requirement: Evidence preserves static claim limits

r[octet.message_boundary.evidence]

The architecture receipt MUST bind message-boundary policy, owner declarations, normalized type coverage, transition paths, findings, unknowns, contributing mechanism refs, waivers, target, features, toolchain, status, and non-claims with BLAKE3 identities.

The receipt MUST state that static conformance does not prove canonical serialization, live and simulated parity, scheduler completeness, deterministic replay, delivery, authority, correctness, or release readiness.

#### Scenario: Receipt verifies without rerunning source analysis

- GIVEN a complete stored architecture receipt and its bound artifacts
- WHEN artifact verification runs
- THEN Octet MUST recompute stored identities and links
- AND it MUST reject missing, changed, stale, or contradictory message-boundary artifacts.

#### Scenario: Static result is promoted to runtime proof

- GIVEN a consumer labels a passing message-boundary receipt as deterministic replay evidence
- WHEN evidence role validation runs
- THEN validation MUST reject the role mismatch.

### Requirement: Consumer surfaces expose the mechanism

r[octet.message_boundary.consumer]

Octet MUST document and template project-selected message-boundary policy, provider-handle identities, staged adoption modes, remediation, evidence output, and non-claims without enabling the mechanism for unconfigured consumers.

#### Scenario: Existing consumer has no mechanism

- GIVEN a consumer has an architecture policy without a message-boundary declaration
- WHEN `cargo octet check` runs
- THEN existing policy behavior MUST remain unchanged
- AND output MUST make no message-boundary compliance claim.

#### Scenario: Consumer selects strict admission

- GIVEN a consumer selects the mechanism in `admission` mode
- WHEN lint, architecture, and effect closure complete
- THEN the parent result MUST identify each phase separately
- AND any failed or unknown required message-boundary fact MUST block admission.

### Requirement: Verification covers accepted and rejected boundaries

r[octet.message_boundary.verification]

Octet MUST include positive and negative compiler-backed fixtures for direct, aliased, generic, associated-type, async, callback, borrowed-buffer, channel, shared-state, feature, target, provider, waiver, and receipt behavior.

#### Scenario: Valid adapter boundary passes

- GIVEN a fixture keeps the live handle in an adapter and passes an owned message to a pure state transition
- WHEN the complete architecture gate runs
- THEN the message-boundary mechanism MUST pass with no handle-leak finding.

#### Scenario: Hidden handle fixture fails

- GIVEN a fixture hides a live runtime handle behind aliases and nested generics in a core message
- WHEN the complete architecture gate runs
- THEN the mechanism MUST fail and identify the resolved hidden handle.
