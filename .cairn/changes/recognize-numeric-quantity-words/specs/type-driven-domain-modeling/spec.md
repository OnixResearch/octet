# Type-driven Domain Modeling Delta

## ADDED Requirements

### Requirement: Quantity words are complete identifier components

r[octet.type_driven_domain_modeling.quantity_words] `numeric_units` MUST recognize its quantity vocabulary as complete underscore-delimited ASCII-case-insensitive identifier components. Unrelated words containing the same substrings MUST NOT become quantities. Existing raw-numeric type, nominal-type, skip-name, unit-suffix and severity behavior MUST remain unchanged.

#### Scenario: Real quantities retain diagnostics

- GIVEN a raw numeric `network_port`, `request_timeout`, or `buffer_size` without an admitted unit suffix
- WHEN the actual compiler lint inspects the declaration
- THEN it MUST report the existing numeric-units diagnostic.

#### Scenario: Counter and transport names do not imply ports

- GIVEN raw numeric import/export counts or transport/support names in fields, parameters, or local bindings
- WHEN the actual compiler lint inspects the declaration
- THEN an embedded `port` substring MUST NOT produce a numeric-units diagnostic.

#### Scenario: Explicit units and nominal types retain admission

- GIVEN a declared time quantity with a recognized suffix or a resolved nominal unit type
- WHEN the lint inspects the declaration
- THEN it MUST retain the existing acceptance without requiring a duplicate suffix.

#### Scenario: Correction does not weaken downstream policy

- GIVEN HWR's unchanged deny-all policy and original count fields
- WHEN the corrected actual owning library runs
- THEN import/export substrings MUST NOT produce numeric-units findings; every independent lint and admission obligation MUST remain enabled.
