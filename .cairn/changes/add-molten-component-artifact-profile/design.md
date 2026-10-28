# Design: Add a Molten component artifact profile

## Decisions

### Decision: A separate profile, not a baseline edit

`portable-component-baseline` remains import-free. `molten-component-baseline`
admits exactly the declared type import `component::preserves-payload::Type(Eq(0))`
and bulk memory, with baseline limits, severity, and recorded-only role.

### Decision: Granular proposal vocabulary

`bulk-memory` maps to `WasmFeatures::BULK_MEMORY`, so the profile does not admit
all of Wasm 2 (SIMD, reference types, multi-value). Unknown proposal tokens still
fail profile validation.

### Decision: Deterministic projection

The generated `profiles.json` is produced by `nickel export --format json` and
checked against the Nickel source.

## Claim Boundary

A profile admits static artifact shape only. It does not prove execution,
component correctness, runtime authority, or release eligibility.
