# Strict Rust Profiles Delta

## ADDED Requirements

### Requirement: Reviewed exact core values

r[octet.strict_rust_profiles.reviewed_core_values] `concrete_construction_in_core` MUST accept a default-empty `core_value_provider_types` configuration of exact compiler-resolved ADT definition paths. A selected external ADT MUST classify as a project-reviewed value for this lint only. Every unselected external ADT MUST retain the existing conservative constructor diagnostic.

#### Scenario: Selected value and alias

- GIVEN a selected external value ADT and an alias for that same compiler definition
- WHEN direct new/default constructors in declared core scope return either spelling
- THEN neither constructor MUST emit this construction diagnostic.

#### Scenario: Same-crate sibling remains unclassified

- GIVEN a selected ADT and a different unselected ADT from the same dependency
- WHEN the unselected ADT is constructed directly in core scope
- THEN its constructor MUST still emit exactly one finding.

#### Scenario: Missing and nonexact facts remain conservative

- GIVEN absent value facts, a crate-only name, a prefix or a path with extra segments
- WHEN an external direct constructor is inspected
- THEN no fact MUST match the external result ADT unless all compiler definition-path segments are equal.

#### Scenario: Claims remain bounded

- GIVEN an exact reviewed-value declaration suppresses this heuristic finding
- WHEN the result is reported
- THEN it MUST NOT be promoted to semantic purity, provider identity verification, effect closure, whole-program correctness or release eligibility.

## MODIFIED Requirements

### Requirement: Core construction classification follows ownership

r[octet.strict_rust_profiles.core_construction_precision] `concrete_construction_in_core` MUST inspect direct constructor calls and distinguish local, standard and explicitly reviewed exact value ADTs from unclassified external concrete construction.

#### Scenario: Unclassified external constructor is reported once

r[octet.strict_rust_profiles.core_construction_precision.external]

- GIVEN a declared core scope directly calls new/default for an unclassified external ADT
- WHEN the lint runs
- THEN Octet MUST emit exactly one diagnostic for that constructor call.

#### Scenario: Local and standard constructors pass

r[octet.strict_rust_profiles.core_construction_precision.local]

- GIVEN a declared core scope constructs a local domain DTO or core/alloc/std value
- WHEN the lint runs
- THEN Octet MUST NOT report this construction diagnostic.

#### Scenario: Nested parents do not duplicate findings

r[octet.strict_rust_profiles.core_construction_precision.no_duplicates]

- GIVEN a direct external constructor appears inside a return, assignment, condition or larger expression
- WHEN the lint walks parent expressions
- THEN only the direct constructor call MUST be eligible for a diagnostic.
