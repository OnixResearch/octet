## ADDED Requirements

### Requirement: Conservative primitive HIR const subset

r[octet.purity_effect_lints.missing_const_fn.hir_subset] `missing_const_fn` MUST require a free-function body in a conservative primitive-valued HIR subset in addition to existing source guardrails. Calls, method calls, closures, type-dependent operators, methods, and unsupported constructs MUST NOT be treated as const-compatible merely because source contains arithmetic, comparisons, or match tokens. This is advisory subset evidence, not a full compiler const proof.

#### Scenario: Primitive arithmetic remains reportable

- GIVEN a private free helper multiplying a primitive integer by a constant
- WHEN the const lint runs
- THEN it emits the existing advisory diagnostic.

#### Scenario: Non-const parsing and calls remain clean

- GIVEN private helpers that compare values but also call string iterators, closures, split_once, parse, arbitrary functions, or overloaded addition
- WHEN the const lint runs
- THEN it emits no const advisory for those helpers.

#### Scenario: Trait method remains clean

- GIVEN a trait implementation method whose body performs primitive arithmetic
- WHEN the const lint runs
- THEN it emits no const advisory for that method.
