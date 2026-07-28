# Type Driven Domain Modeling Specification Delta

## ADDED Requirements

### Requirement: Nominal domain policy is typed

r[octet.type_driven_domain_modeling.nominal_policy] Octet MUST define a Nickel-authored policy for enforced nominal domains. Each domain MUST declare its raw representation, core scopes, compatibility scopes, wrapper kind, and constructor policy.

#### Scenario: Declared nominal policy is accepted

r[octet.type_driven_domain_modeling.nominal_policy.accepted]
- GIVEN a policy declares unique domains, supported raw representations, bounded core scopes, explicit compatibility scopes, and valid constructor rules
- WHEN Octet exports and loads the policy
- THEN the generated runtime data MUST be deterministic
- AND the checker MUST apply the declared scope without ambient repository discovery.

#### Scenario: Invalid nominal policy fails closed

r[octet.type_driven_domain_modeling.nominal_policy.fixtures]
- GIVEN a policy has duplicate domains, unknown representations, unsafe overlapping scopes, missing required exceptions, or invalid constructor rules
- WHEN policy export or checker startup runs
- THEN Octet MUST fail with deterministic policy diagnostics
- AND it MUST NOT continue with partial nominal enforcement.

### Requirement: Primitive domain aliases are reported

r[octet.type_driven_domain_modeling.primitive_alias] Octet MUST report a declared core domain whose type alias resolves directly to its configured raw representation.

#### Scenario: Primitive domain alias is rejected

r[octet.type_driven_domain_modeling.primitive_alias.raw]
- GIVEN `UnitId` is a declared string-backed domain in a checked core scope
- WHEN the source declares `type UnitId = String`
- THEN `primitive_domain_alias` MUST report the alias with the configured domain identity.

#### Scenario: Generic nominal alias is accepted

r[octet.type_driven_domain_modeling.primitive_alias.nominal]
- GIVEN `UnitId` aliases a nominal generic instantiation such as `DomainId<UnitRole>`
- WHEN resolved-type matching runs
- THEN Octet MUST NOT classify the alias as a primitive alias.

### Requirement: Raw domain values are excluded from core APIs

r[octet.type_driven_domain_modeling.raw_domain_value] Octet MUST report configured raw domain representations in declared core fields, parameters, results, enum payloads, and map keys unless an explicit compatibility scope applies.

#### Scenario: Raw core value is rejected

r[octet.type_driven_domain_modeling.raw_domain_value.core]
- GIVEN a declared `SourceId` domain uses `String` as its wire representation
- WHEN a checked core API exposes `source_id: String`
- THEN `raw_domain_value` MUST report the raw core boundary.

#### Scenario: Compatibility DTO remains accepted

r[octet.type_driven_domain_modeling.compatibility_exceptions]
- GIVEN a declared wire, FFI, generated, vendored, or protocol DTO scope uses the required raw representation
- WHEN nominal enforcement checks that scope
- THEN the hard finding MUST be suppressed
- AND the policy MUST identify the adapter boundary into typed core data.

### Requirement: Refined wrappers prevent unchecked construction

r[octet.type_driven_domain_modeling.wrapper_contract] Refined domains in enforced scope MUST keep invariant-bearing inner fields private and MUST use fallible construction from untrusted primitive input.

#### Scenario: Refined wrapper bypass is rejected

r[octet.type_driven_domain_modeling.wrapper_contract.bypass]
- GIVEN a refined digest, path, token, secret, or bounded identifier has a public inner field or unrestricted primitive `From` conversion
- WHEN `newtype_invariant_bypass` checks the wrapper
- THEN Octet MUST report the unchecked construction path.

#### Scenario: Explicit checked wrapper passes

r[octet.type_driven_domain_modeling.wrapper_contract.checked]
- GIVEN a refined wrapper has a private field, fallible parser or `TryFrom` conversion, and explicit borrowed accessors
- WHEN wrapper checking runs
- THEN Octet MUST accept the wrapper shape.

### Requirement: Symptom lints recognize owned semantic primitives

r[octet.type_driven_domain_modeling.symptom_lints] `ambiguous_params` and `numeric_units` MUST recognize configured owned primitive-like types, associated functions, methods, parameters, and fields without reporting values whose resolved nominal type already carries the distinction.

#### Scenario: Repeated raw API values are reported

r[octet.type_driven_domain_modeling.symptom_lints.raw]
- GIVEN a public constructor accepts repeated `String` or path values that can be exchanged without a compiler error
- WHEN `ambiguous_params` checks the signature
- THEN it MUST report the exchangeable raw parameters.

#### Scenario: Typed units suppress naming findings

r[octet.type_driven_domain_modeling.symptom_lints.typed]
- GIVEN a field or parameter uses a configured duration or byte-count newtype
- WHEN `numeric_units` checks its name
- THEN it MUST accept the type-carried unit without requiring a duplicate suffix.

### Requirement: Compile-time role separation has fixture evidence

r[octet.type_driven_domain_modeling.compile_time_separation] Octet MUST include compile-pass and compile-fail fixtures proving that configured marker-domain aliases accept same-domain values and reject cross-domain substitution.

#### Scenario: Cross-domain substitution does not compile

r[octet.type_driven_domain_modeling.compile_time_separation.negative]
- GIVEN `UnitId` and `SourceId` use distinct marker instantiations over the same representation
- WHEN a fixture passes `SourceId` to an API that requires `UnitId`
- THEN the fixture MUST fail compilation for a type mismatch.

### Requirement: Nominal enforcement is promoted in stages

r[octet.type_driven_domain_modeling.staged_promotion] Octet MUST separate heuristic inventory from exact-policy enforcement and MUST require positive, negative, and compatibility evidence before adding nominal checks to a strict profile.

#### Scenario: Heuristics do not become hard policy

r[octet.type_driven_domain_modeling.staged_promotion.heuristics]
- GIVEN an unregistered field name matches an identity, digest, token, path, unit, or revision suffix
- WHEN inventory mode runs
- THEN Octet CAN report a review candidate
- AND strict-profile denial MUST NOT use that heuristic without an exact policy row.

#### Scenario: Strict promotion uses reviewed evidence

r[octet.type_driven_domain_modeling.staged_promotion.evidence]
- GIVEN fixture and pilot evidence demonstrates accepted core types, rejected category errors, and valid compatibility exceptions
- WHEN maintainers promote a nominal check
- THEN the strict profile MUST bind the reviewed policy and evidence identity.

### Requirement: Nominal enforcement claims remain bounded

r[octet.type_driven_domain_modeling.docs] Octet documentation and diagnostics MUST state that nominal checks prove configured source-shape conformance only.

#### Scenario: Enforcement claim remains bounded

r[octet.type_driven_domain_modeling.final_checks]
- GIVEN nominal checks pass for a selected scope
- WHEN Octet reports the result
- THEN it MUST NOT claim runtime correctness, authority, secrecy, evidence truth, compiler correctness, or release eligibility.
