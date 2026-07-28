# Tasks: Enforce nominal domain types

## Phase 1: Policy and pure classification

- [x] [serial] I1 Add a typed Nickel nominal-domain catalog with exact raw representations, core scopes, compatibility scopes, wrapper kinds, and constructor policies. r[octet.type_driven_domain_modeling.nominal_policy]
- [x] [serial] I2 Export `standards/nominal-domains/generated/catalog.json`. Register it in the config and receipt registry and add a Nix freshness rail. r[octet.type_driven_domain_modeling.nominal_policy]
- [x] [parallel] I3 Add positive and negative policy fixtures for duplicate domains, unknown representations, overlapping scopes, missing exceptions, and invalid constructor policy. r[octet.type_driven_domain_modeling.nominal_policy.fixtures]
- [x] [serial] I4 Implement pure descriptor and policy classification helpers for aliases, values, wrappers, and scope exceptions. r[octet.type_driven_domain_modeling.nominal_policy] r[octet.type_driven_domain_modeling.wrapper_contract]

## Phase 2: Compiler lints

- [x] [serial] I5 Implement and register `primitive_domain_alias` with resolved-type matching and exact-policy diagnostics. r[octet.type_driven_domain_modeling.primitive_alias]
- [x] [serial] I6 Implement and register `raw_domain_value` for fields, parameters, results, enum payloads, and map keys. r[octet.type_driven_domain_modeling.raw_domain_value]
- [x] [serial] I7 Implement and register `newtype_invariant_bypass` for public refined fields and unchecked primitive conversions. r[octet.type_driven_domain_modeling.wrapper_contract]
- [x] [parallel] I8 Extend `ambiguous_params` and `numeric_units` without changing accepted typed-domain behavior. r[octet.type_driven_domain_modeling.symptom_lints]

## Phase 3: Fixtures and compatibility

- [x] [parallel] V1 Add passing UI fixtures for private newtypes, generic marker aliases, unit types, explicit accessors, and wire adapters. r[octet.type_driven_domain_modeling.primitive_alias] r[octet.type_driven_domain_modeling.compatibility_exceptions]
- [x] [parallel] V2 Add failing UI fixtures for primitive aliases, raw core values, invariant bypasses, repeated raw parameters, and unitless fields. r[octet.type_driven_domain_modeling.raw_domain_value] r[octet.type_driven_domain_modeling.wrapper_contract] r[octet.type_driven_domain_modeling.symptom_lints]
- [x] [parallel] V3 Add compile-fail fixtures that reject cross-domain substitution and compile-pass fixtures for same-domain use. r[octet.type_driven_domain_modeling.compile_time_separation]
- [x] [parallel] V4 Add compatibility fixtures for Serde DTOs, FFI, generated code, generic aliases, and unchanged JSON projections. r[octet.type_driven_domain_modeling.compatibility_exceptions]

## Phase 4: Rollout and promotion

- [x] [serial] I9 Add frozen Valence and Mantle pilot receipts under `evidence/pilots/`. Use schema `octet-nominal-domain-inventory/v1` without live sibling dependencies. r[octet.type_driven_domain_modeling.staged_promotion]
- [x] [serial] I10 Add inventory, warning, and deny modes with heuristic suffixes restricted to inventory output. r[octet.type_driven_domain_modeling.staged_promotion]
- [x] [serial] I11 Promote exact-policy checks into reviewed strict profiles only after saved fixture and pilot evidence meets the signal threshold. r[octet.type_driven_domain_modeling.staged_promotion]
- [x] [parallel] V5 Update lint metadata, lint reference, strict-profile docs, downstream rollout guidance, and claim boundaries. r[octet.type_driven_domain_modeling.docs]
- [x] [serial] V6 Run policy freshness, focused unit tests, UI tests, consumer fixtures, strict-profile checks, Cairn validation, and all change gates. r[octet.type_driven_domain_modeling.final_checks]

## Verification coverage

- `Scenario: Declared nominal policy is accepted` -> I1, I2, I3, I4
- `Scenario: Invalid nominal policy fails closed` -> I3, V6
- `Scenario: Primitive domain alias is rejected` -> I5, V2
- `Scenario: Generic nominal alias is accepted` -> I5, V1
- `Scenario: Raw core value is rejected` -> I6, V2
- `Scenario: Compatibility DTO remains accepted` -> I6, V4
- `Scenario: Refined wrapper bypass is rejected` -> I7, V2
- `Scenario: Repeated raw API values are reported` -> I8, V2
- `Scenario: Typed units suppress naming findings` -> I8, V1
- `Scenario: Cross-domain substitution does not compile` -> V3
- `Scenario: Heuristics do not become hard policy` -> I10, V4
- `Scenario: Strict promotion uses reviewed evidence` -> I9, I10, I11, V6
- `Scenario: Enforcement claim remains bounded` -> V5, V6
