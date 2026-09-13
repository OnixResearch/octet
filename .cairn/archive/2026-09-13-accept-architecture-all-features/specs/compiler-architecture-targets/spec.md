## ADDED Requirements

### Requirement: Cargo-resolved all-feature equivalence
r[octet.architecture.features.equivalence]

The collector MUST accept `--all-features` only when a bounded locked Cargo plan equals the plan for the explicit policy row.
The comparison MUST preserve complete unit descriptions, resolved features, dependency edges, and selected roots.
Unit indices and collection order MUST NOT determine equivalence.

#### Scenario: Complete reviewed row
- GIVEN a policy row that resolves to every feature and target selected by the invocation
- WHEN the collector receives `--all-targets --all-features`
- THEN it accepts the selection and binds compiler units to the actual Cargo-resolved features.

#### Scenario: Omitted feature or dependency
- GIVEN a policy row that omits a selected feature, feature-gated target, optional dependency, or dependency edge
- WHEN the actual and declared plans differ
- THEN the collector rejects the invocation before publishing a collector context or clean architecture evidence.

### Requirement: Feature selection fails closed
r[octet.architecture.features.rejection]

All-feature support MUST retain exact target, default-feature, production-package, and compiler-shard checks.
It MUST reject mixed or repeated feature selectors and unsupported shorthand.
The comparison MUST reject malformed, incomplete, or ambiguous graph facts.

#### Scenario: Ambiguous or mismatched input
- GIVEN mixed feature selectors, repeated selectors, wrong defaults, wrong targets, or missing graph references
- WHEN collection starts
- THEN it returns an integration error without a clean architecture result.

#### Scenario: Missing actual compiler unit
- GIVEN equivalent accepted plans and a missing expected compiler shard
- WHEN the collector checks coverage
- THEN coverage remains incomplete and the invocation fails.

### Requirement: All-feature publication evidence
r[octet.architecture.features.evidence]

Publication MUST include actual compiling all-target controls for explicit and all-feature invocations.
It MUST include a full deny-all consumer pass and a rejected under-declared policy row.
It MUST retain focused tests, strict Clippy, formatting, and lifecycle evidence.
Planner-only, warning-only, or policy-disabled results MUST NOT establish all-feature acceptance.

#### Scenario: Executed positive and negative controls
- GIVEN current runner and collector artifacts with an active architecture policy
- WHEN the full deny-all hook checks complete and incomplete explicit policy rows with `--all-features`
- THEN the complete row passes and the incomplete row fails before compiler acceptance.

#### Scenario: Evidence lacks actual compilation
- GIVEN only source inspection or successful unit planning
- WHEN publication is requested
- THEN acceptance remains incomplete.
