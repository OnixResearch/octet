## MODIFIED Requirements

### Requirement: Nominal Type Boundary Guidance [r[nominal-type-boundary-guidance]]

The system MUST document when Rust code should use nominal newtypes instead of interchangeable primitive or structural types for IDs, units, secrets, capability tokens, protocol states, and authority-bearing values.

For declared type-state protocol surfaces, the system MUST export structural checker obligations that reject configured raw state aliases and raw transition bypasses while accepting nominal marker-state APIs with visible transition methods.

#### Scenario: Compliant nominal marker-state API passes [r[nominal-type-boundary-guidance-scenario-compliant-typestate-passes]]

- GIVEN a generated policy artifact declares type-state source scopes, required state marker names, and required transition method names
- WHEN `tigerstyle-standards check --policy-artifacts-dir` scans a compliant source root with nominal marker states and transition methods
- THEN the type-state structural policy emits no diagnostics for that compliant source root

#### Scenario: Raw protocol bypass fails structurally [r[nominal-type-boundary-guidance-scenario-raw-protocol-bypass-fails]]

- GIVEN a generated policy artifact declares raw state alias and raw transition bypass patterns
- WHEN `tigerstyle-standards check --policy-artifacts-dir` scans a declared source root containing one of those raw bypasses
- THEN the type-state structural policy emits an error diagnostic naming the configured bypass pattern and exits nonzero when hard policies fail

### Requirement: Nickel Mechanism Contracts [r[nickel-mechanism-contracts]]

The system MUST define mechanism-specific Nickel contracts for crate boundaries, capability APIs, bounded primitives, type-state protocols, proof rails, and seed corpora.

Mechanism contracts MUST include the fields needed by Rust checkers to validate the corresponding structural or verified claim without inferring policy intent from prose.

For type-state mechanisms, Nickel contracts MUST support deterministic exported fields for source scopes, raw state alias patterns, raw transition patterns, required state marker names, and required transition method names.

#### Scenario: Type-state obligations export deterministically [r[nickel-mechanism-contracts-scenario-type-state-obligations-export]]

- GIVEN the policy registry declares a `type-state` mechanism for nominal type boundaries
- WHEN generated policy artifacts are exported from the Nickel registry
- THEN the policy JSON and checker config include the configured type-state source scopes, raw bypass patterns, state marker names, and transition method names

### Requirement: Structural Mechanism Validation [r[structural-mechanism-validation]]

The system MUST validate claimed structural enforcement mechanisms before accepting a policy as structurally enforced.

Structural validation MUST cover at least no-std core crates, shell-only side effects, approved capability constructors, bounded primitive use, type-state protocol surfaces, and crate dependency direction.

For bounded queue policies, structural validation MUST reject configured unbounded queue constructors in selected source roots and MUST accept only approved bounded constructors or wrappers with explicit capacity and backpressure witnesses.

For effect capability policies, structural validation MUST reject configured ambient authority APIs in declared core source scopes while accepting shell-owned constructors and injected capability or effect-data APIs.

For type-state protocol policies, structural validation MUST reject configured raw state aliases and raw transition shortcuts in declared source scopes while accepting nominal marker-state wrappers with declared transition methods.

#### Scenario: Focused Tigerstyle checks pass [r[structural-mechanism-validation-scenario-focused-typestate-checks-pass]]

- GIVEN the type-state structural checker, fixtures, policy registry, and generated artifact export are changed together
- WHEN the focused Rust, Nickel/Nix, OpenSpec, docs, and whitespace checks run
- THEN every focused check exits successfully and the saved transcript names the commands used
