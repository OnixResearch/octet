# Design: Add a Molten component artifact profile

## Decisions

### Decision: A separate profile, not a baseline edit

`portable-component-baseline` remains import-free. `molten-component-baseline`
admits exactly the declared type import `component::preserves-payload::Type(Eq(0))`
and bulk memory plus padded indirect calls, with baseline limits, severity, and recorded-only role.

### Decision: Granular proposal vocabulary

`bulk-memory` maps to `WasmFeatures::BULK_MEMORY`, so the profile does not admit
all of Wasm 2 (SIMD, reference types, multi-value). Unknown proposal tokens still
fail profile validation.

`call-indirect-overlong` maps only to `WasmFeatures::CALL_INDIRECT_OVERLONG`.
The actual LLVM-linked guest uses a padded LEB zero table immediate: without
that feature wasmparser reports `zero byte expected` at offset `0x491`.
This narrow encoding feature does not enable reference types, multi-value or
SIMD. A real byte-validation regression accepts bulk memory and padded calls,
rejects reference-type/SIMD instructions, and keeps the portable profile denying
both newly declared features. The profile identity and registry identity change;
consumers must refresh their pinned cohorts, never relabel earlier receipts.

### Decision: Deterministic projection

The generated `profiles.json` is produced by `nickel export --format json` and
checked against the Nickel source.

## Claim Boundary

A profile admits static artifact shape only. It does not prove execution,
component correctness, runtime authority, or release eligibility.
