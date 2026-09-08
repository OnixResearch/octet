## ADDED Requirements

### Requirement: Explicit Cargo target coverage
r[octet.architecture.targets.coverage]

The collector MUST derive selected compilation units from a bounded locked Cargo plan. It MUST support explicit all-target selection without silently omitting non-library Rust targets. Unsupported modes and ambiguous selected unit keys MUST fail closed.

#### Scenario: Complete target plan
- GIVEN explicit library, binary, test, example, benchmark, proc-macro, and build-script compilation units
- WHEN the target plan is accepted
- THEN each selected Rust unit has an expected compiler identity.

#### Scenario: Ambiguous or unsupported plan
- GIVEN duplicate observation keys, an unknown graph version, or an unsupported unit mode
- WHEN planning runs
- THEN no clean coverage result is emitted.

### Requirement: Compiler observations select exact units
r[octet.architecture.targets.binding]

Collector context v2 MUST bind the source root, test configuration, target triple, and resolved features. The collector MUST reject conflicting or unexpected observations for selected packages. Missing units MUST remain coverage failures.

#### Scenario: Same crate with distinct test configuration
- GIVEN normal and test compilations with the same crate name
- WHEN their compiler observations arrive
- THEN each matches only its exact expected unit.

#### Scenario: Wrong configuration or omitted unit
- GIVEN a wrong source root, target, feature set, test configuration, or absent expected unit
- WHEN collector or coverage validation runs
- THEN it rejects the mismatch without reusing another unit's evidence.

### Requirement: Verified publication preserves claim limits
r[octet.architecture.targets.evidence]

Publication MUST include compiling positive and negative controls, current tests, strict Clippy, and generated-contract freshness. Compilation failure MUST NOT count as architecture rejection. Library-only or source-only evidence MUST NOT establish all-target compiler acceptance.

#### Scenario: Actual compiler control
- GIVEN a compiled pure or effectful fixture and current collector artifacts
- WHEN the evidence is evaluated
- THEN it records exact unit coverage and observed effects separately from lint findings.

#### Scenario: Missing compiler evidence
- GIVEN only planner tests or source inspection
- WHEN publication is requested
- THEN all-target acceptance remains blocked.
