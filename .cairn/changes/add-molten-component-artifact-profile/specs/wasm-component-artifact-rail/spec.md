# Molten Component Artifact Profile Delta

## ADDED Requirements

### Requirement: Molten profile admits only declared shape

r[octet.molten_profile.profile] Octet MUST define a typed Nickel `molten-component-baseline` profile that admits component kind, the single declared `component::preserves-payload::Type(Eq(0))` import, bulk memory, baseline structural and resource limits, error severity, recorded-only verification role, and explicit non-claims, and MUST consume its deterministic fresh export.

#### Scenario: Admitted bundle passes
- GIVEN the supported Molten component with that type import and bulk memory
- WHEN the profile analyzes exact bytes
- THEN no disallowed-import or disallowed-proposal finding is produced

#### Scenario: Any other import denies
- GIVEN a component importing anything else
- WHEN the profile analyzes it
- THEN the import is a disallowed-import finding

### Requirement: Bulk memory is a granular proposal

r[octet.molten_profile.granular_proposal] The `bulk-memory` proposal token MUST map to the bulk-memory validation feature only, without enabling other Wasm 2 proposals, and unknown proposal tokens MUST fail profile validation.

#### Scenario: Unknown proposal denies
- GIVEN a profile naming `bulk-memory-unknown`
- WHEN registry validation runs
- THEN it reports an unknown proposal

### Requirement: Baseline stays import-free

r[octet.molten_profile.baseline_unchanged] `portable-component-baseline` MUST keep empty allowed imports and only `wasm1` and `component-model` proposals.

#### Scenario: Baseline is unchanged
- GIVEN the existing baseline profile
- WHEN the registry is validated
- THEN it still admits no imports and no bulk memory

### Requirement: Non-claims remain explicit

r[octet.molten_profile.nonclaims] Profile admission of one declared import and bulk memory MUST NOT be presented as admitting other imports, proposals, runtime authority, execution correctness, or release eligibility.

#### Scenario: Evidence carries the added non-claim
- GIVEN a receipt produced with `molten-component-baseline`
- WHEN it is reviewed
- THEN the profile's added admission non-claim is present
