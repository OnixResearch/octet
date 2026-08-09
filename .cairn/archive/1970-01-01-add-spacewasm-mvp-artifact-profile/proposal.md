## Why

Octet's active Wasm artifact rail plans one wasm-tools-backed static path for modules and components. Downstream experiments with SpaceWasm need a narrower artifact profile that can answer whether exact bytes use only the WebAssembly 1.0 subset and resource shape admitted by one reviewed SpaceWasm cohort before any runtime is constructed.

SpaceWasm's repository prose, issue tracker, and current source can drift independently. A generic “valid Wasm” result or matching import list cannot prove that a module will decode, instantiate, or execute in SpaceWasm. Octet should provide stable static facts and profile decisions while remaining execution-free and rejecting runtime/certification overclaims.

## What Changes

- Add a typed Nickel `spacewasm-mvp` artifact profile layered on the general Wasm artifact rail.
- Bind the profile to one exact reviewed SpaceWasm source/materialization cohort and a checked support-matrix projection.
- Require core-module kind and explicitly classify WebAssembly proposals, imports/exports, memories/tables, start functions, `memory.grow`, structural limits, producer metadata, and declared ABI/resources.
- Reject components, WIT/WASI requirements, unsupported post-MVP operators, and profile/resource violations with deterministic findings.
- Keep exact artifact BLAKE3, SpaceWasm cohort/profile identity, and any normalized interface/ABI identity in separate roles.
- Optionally ingest verified Mantle or ChaosControl SpaceWasm reports as recorded-only external evidence without executing the runtime or reinterpreting their semantics.
- Emit stable standalone and Mantle handoff reports that downstream consumers can reverify.
- Add positive and negative generated corpora for the admitted intersection and profile rejection boundaries.

## Impact

- **Surfaces**: Wasm profile registry, support-matrix projection, pure artifact analysis, findings, reports, artifact bundles, external-evidence intake, Valence export, and fixtures.
- **Dependency**: depends on the general `add-wasm-component-artifact-rail` parser/report foundation; it does not add a second parser/runtime to Octet's trusted core.
- **Consumers**: Mantle, ChaosControl, Molten, Animus, and Kamacite may consume the stable facts but must make their own runtime/build/admission decisions.
- **Claims**: passing proves only bounded static conformance to one declared SpaceWasm profile. It does not prove SpaceWasm acceptance, execution, memory safety, semantic equivalence, sandboxing, flight qualification, or release eligibility.
