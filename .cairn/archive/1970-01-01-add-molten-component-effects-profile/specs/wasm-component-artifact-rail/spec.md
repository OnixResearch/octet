# Exact Molten Component Effects Profile Delta

## ADDED Requirements

### Requirement: Metadata scope follows binary nesting

r[octet.molten_effects.metadata_scope] Duplicate custom-section policy MUST compare section names only within the same module/component binary scope and MUST retain deterministic scope-qualified metadata facts.

#### Scenario: Independent nested name sections pass
- GIVEN separate nested binaries each containing one name section
- WHEN a reject-duplicates profile analyzes the component
- THEN no duplicate metadata finding occurs

#### Scenario: Same-scope name duplication denies
- GIVEN a nested binary containing two name sections
- WHEN that profile analyzes the component
- THEN duplicate-custom-section is reported

### Requirement: Internal binding does not erase unresolved authority

r[octet.molten_effects.boundary] Octet MUST distinguish root exports and unresolved imports from imports proven supplied by validated local instantiation, MUST record bound imports separately, and MUST retain unresolved authority from unused or exported reusable definitions. A nested unresolved component import MUST NOT inherit root import allowances.

#### Scenario: Internally supplied adapter passes
- GIVEN a valid local provider instance supplying an adapter's function and table imports
- WHEN an import-free profile analyzes the component
- THEN those adapter imports are recorded as bound and do not become external import denials

#### Scenario: Unused or exported adapter retains imports
- GIVEN the same adapter uninstantiated or exported as a reusable module
- WHEN an import-free profile analyzes the component
- THEN its unresolved imports are denied

### Requirement: Resource evidence counts definitions and instantiations

r[octet.molten_effects.resources] Resource counts MUST exclude proven bound import aliases as independent allocations, MUST multiply definitions by local and enclosing instantiation multiplicity with saturating arithmetic, MUST conservatively count unused definitions once, and MUST enforce retained declaration bounds. Unknown imported/aliased generative implementations MUST fail accounting closed. Absence of a declared maximum MUST NOT be reported as a runtime bound.

#### Scenario: Table alias is not a new table
- GIVEN one defined table supplied to an adapter by import
- WHEN the component is analyzed
- THEN the independent table count is one

#### Scenario: Repeated instantiation exceeds the count bound
- GIVEN a one-table module instantiated twice and a one-table count limit
- WHEN the component is analyzed
- THEN the count is two and a resource-limit finding is reported

#### Scenario: Effects table boundaries deny above limits
- GIVEN the effect profile with two independent tables allowed and the declared element limit
- WHEN table count or declaration elements exceed the respective limit
- THEN resource-limit or table-limit is reported, while the exact boundary is admitted

### Requirement: Effects profile admits exact structural surface only

r[octet.molten_effects.profile] Nickel MUST define separate `molten-component-effects-v1` with `component_interface = "structural-v1"`, exactly required root preserves-payload equality(list<u8>) type bookkeeping and effect-request function imports, and sole required invoke export. Both functions MUST have synchronous `(list<u8>) -> result<list<u8>,string>` signatures. The profile MUST allow only wasm1/component-model/bulk-memory/call-indirect-overlong, two imports, one export, one memory and two independent tables, retain remaining baseline limits and denial postures, and remain recorded-only. Both existing published profile policies MUST remain unchanged.

#### Scenario: Padded indirect encoding stays narrowly scoped
- GIVEN canonical or overlong LEB table-zero call-indirect encoding
- WHEN the effect profile analyzes it
- THEN that encoding is admitted without enabling general reference types or SIMD
- AND undeclared nonzero table indices and reference operations are denied

#### Scenario: Canonical type indexing is immaterial
- GIVEN identical admitted structural signatures with different local type indices
- WHEN the exact bytes are analyzed
- THEN both pass with equal interface identities and different exact-byte identities

#### Scenario: Wrong or undeclared boundary denies
- GIVEN missing required imports/exports, extra WASI/product functions, or wrong argument/result/payload types
- WHEN the effects profile analyzes the component
- THEN disallowed-import, disallowed-export, or abi-mismatch findings deny it

#### Scenario: Existing profiles remain closed
- GIVEN the effect surface with its product function import
- WHEN either existing baseline profile analyzes it
- THEN the effect-request import is denied

### Requirement: Identity and authority remain separate

r[octet.molten_effects.identity] Receipts MUST retain exact original byte, selected canonical profile, full registry, and cohort identities. Corrected boundary interface identities MUST use the v2 domain. Optional profile additions MUST preserve pre-existing profile DTO identities when absent. Structural surface admission MUST NOT authenticate a package/world name or transfer canonical Preserves, effect/capability, semantic, runtime, provenance, or release authority to Octet.

#### Scenario: Parent replays the real guest and receipts
- GIVEN the retained normalized supported effect guest and generated profile registry
- WHEN the parent integrated verification analyzes the exact bytes and verifies the artifact bundle
- THEN the new profile passes static analysis with the exact bytes/profile identities retained and explicit recorded-only non-claims
- AND no runtime execution or semantic correctness claim follows from that pass

