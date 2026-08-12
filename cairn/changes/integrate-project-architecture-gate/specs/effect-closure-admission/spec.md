# Effect Closure Admission Specification Delta

## MODIFIED Requirements

### Requirement: Two-stage closure CLI

r[octet.effect_closure.cli] Effect closure MUST run only after its required direct strict check and any activated project architecture gate succeed.

#### Scenario: Direct check, architecture gate, and closure are admitted

r[octet.effect_closure.cli.admitted]

- GIVEN the strict check passes, active project architecture gate passes, function addresses match, calls resolve, role edges are allowed, and closure effects satisfy policy
- WHEN `admission` mode or the explicit closure command runs
- THEN Octet MUST emit an admitted effect-closure bundle and identified parent evidence

#### Scenario: Direct or architecture failure blocks closure

r[octet.effect_closure.cli.check_failure]

- GIVEN the strict check fails or an activated project architecture gate blocks
- WHEN closure orchestration runs
- THEN Octet MUST NOT execute or claim an admitted closure
- AND status evidence MUST identify the failed prerequisite phase

#### Scenario: Explicit closure command has no project policy

- GIVEN a project selects the existing explicit closure command without a project architecture-policy pointer
- WHEN the command runs
- THEN existing strict-profile and closure behavior MUST remain compatible
- AND no project architecture claim MUST be inferred

### Requirement: Compatibility and claim boundaries

r[octet.effect_closure.boundaries] Closure admission MUST remain opt-in and retain its conservative selected-source-token claim boundary when composed with project architecture evaluation.

#### Scenario: Existing check remains unchanged

r[octet.effect_closure.boundaries.default]

- GIVEN an operator invokes `cargo octet check` without project architecture activation
- WHEN the integrated architecture feature is installed
- THEN existing check arguments, lint behavior, address behavior, closure behavior, and artifact schemas MUST remain compatible

#### Scenario: Evidence does not overclaim

r[octet.effect_closure.boundaries.non_claims]

- GIVEN a parent architecture receipt links an effect-closure receipt
- WHEN output describes admission
- THEN each receipt MUST preserve its checker, inputs, evidence tier, assumptions, and non-claims
- AND their combination MUST NOT claim complete MIR closure, runtime determinism, whole-program correctness, or release eligibility