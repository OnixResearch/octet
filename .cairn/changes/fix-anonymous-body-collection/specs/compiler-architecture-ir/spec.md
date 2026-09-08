# Compiler architecture IR delta

## ADDED Requirements

### Requirement: Body-owner queries match compiler kinds

r[octet.compiler_architecture_ir.body_owner_queries] The collector MUST classify each fallback body owner from its compiler definition kind. It MUST query a function signature only for a function definition. Anonymous and inline constant bodies MUST retain private item and call facts without an unsupported visibility query.

#### Scenario: Anonymous constant contains a resolved call

- GIVEN an admitted source contains a function call inside an anonymous const-generic argument
- WHEN the real compiler collector visits that body
- THEN collection MUST complete without a signature-query panic
- AND the constant body MUST have no signature and MUST retain its resolved call fact

#### Scenario: Inline constant contains a resolved call

- GIVEN an admitted source contains a function call inside an inline constant
- WHEN the real compiler collector visits that body
- THEN collection MUST complete without a visibility-query panic
- AND the body MUST be private and MUST have no signature

#### Scenario: Function and closure evidence remains distinct

- GIVEN a fixture contains ordinary functions and closure calls beside anonymous bodies
- WHEN collection runs
- THEN functions MUST retain signatures
- AND closure and constant owners MUST NOT become function facts

#### Scenario: A compiler observation is missing

- GIVEN the fixture compiles and produces its expected compiler shards
- WHEN one shard is removed before normalization
- THEN the normalizer MUST reject complete coverage
