# Compiler architecture body owners

## ADDED Requirements

### Requirement: Caller identities preserve compiler body kinds

r[octet.compiler_architecture_ir.body_owner_kinds]

The collector MUST preserve the compiler definition kind when it discovers a call's body owner.
It MUST request a function signature only for a function kind.
Inline and anonymous constant owners MUST retain constant item identities and their observations, without function signatures.
Lazy discovery MUST use the same item classification as existing local identity calculation.
Expression body owners MUST retain private visibility without a named-item visibility query.

#### Scenario: Constant and static bodies contain calls

- GIVEN named, associated, inline, anonymous, or macro-generated constants and static initializers contain resolved calls
- WHEN the selected compiler collects those calls
- THEN every call MUST reference an item with the matching compiler kind
- AND non-function owners MUST NOT acquire function signatures or disappear from the observation set

#### Scenario: Functions and closures coexist with constants

- GIVEN functions, methods, closures, and constant bodies share selected production units
- WHEN collection and normalization run
- THEN ordinary function signatures and existing closure handling MUST remain intact
- AND item references, unit memberships, and coverage MUST remain valid

#### Scenario: Expanded state contains a forbidden production effect

- GIVEN a thread-local initializer contains an inline constant and a nested production body contains a filesystem operation
- WHEN the compiler collector and project gate run
- THEN collection MUST NOT crash on a function-signature query for the constant
- AND the production effect MUST remain visible and rejected in each observing production unit
- AND omitted compiler shards MUST still reject complete-coverage claims
