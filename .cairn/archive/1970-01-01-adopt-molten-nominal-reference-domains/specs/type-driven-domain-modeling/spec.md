## ADDED Requirements

### Requirement: Molten reference adoption is exact and producer-owned

r[octet.type_driven_domain_modeling.molten_adoption.catalog] Octet MUST register
the 15 `molten.nominal-reference.*` domains and their alias/qualified-field
mapping documented in `docs/type-driven-domain-modeling.md` in the existing
central Nickel catalog. Rows MUST use raw `string`, `marker-generic`, and
`private-fallible`, the exact Molten core file, and required explicit wire
compatibility. Runtime policy MUST remain the source-owned embedded export.
Both Nickel and Rust admission MUST accept the checked generic constructor pair,
retain marker-only support, and reject marker-generic/private-infallible.

#### Scenario: Exact Molten catalog

- GIVEN the 15 reviewed alias and field mappings
- WHEN the producer generates the nominal catalog
- THEN the generated export MUST preserve exact physical source selectors
- AND it MUST NOT replace them with logical migration labels or broad prefixes.
- AND checked generic construction MUST be admitted without admitting unchecked
  generic construction or claiming generic constructor semantic validation.

### Requirement: Local macro aliases enforce configured primitive denial

r[octet.type_driven_domain_modeling.molten_adoption.macros] For local macro aliases,
Octet MUST use the compiler-resolved type and the real user callsite to enforce
exact configured core alias policy. Compatibility MUST remain admitted;
heuristic-only names and external macro expansions MUST remain excluded.

#### Scenario: Local macro String regression

- GIVEN an exact registered alias emitted by Molten's unchanged local domain macro
- WHEN its resolved type changes from a nominal generic wrapper to String
- THEN deny-mode checking MUST fail at the configured core user callsite
- AND compiler UI and real consumer mutation evidence MUST identify that alias.

#### Scenario: Direct core String regression

- GIVEN a registered core struct field from any of the 15 domains
- WHEN it contains String directly or through supported Option storage
- THEN deny-mode checking MUST fail rather than admitting primitive core storage.

### Requirement: Compatibility and evidence limits remain explicit

r[octet.type_driven_domain_modeling.molten_adoption.compatibility] Octet MUST admit
nominal generic aliases/core fields and explicit raw wire-compatible shapes.
Checks MUST NOT imply borrowed primitive traversal, generic constructor validity,
whole-consumer migration, authority, runtime correctness, or release eligibility.
Consumer pins MUST identify an approved immutable available provider cohort.

#### Scenario: Nominal generic compatibility

- GIVEN typed local macro aliases and registered fields or raw shapes in declared compatibility
- WHEN deny-mode compiler UI checks run
- THEN those shapes MUST have no nominal-domain diagnostics.

#### Scenario: Scope and non-claims

- GIVEN only local unpublished provider changes and source-shape evidence
- WHEN consumer integration readiness is assessed
- THEN no accepted immutable-pin, sync/archive, runtime, or release claim is justified.
