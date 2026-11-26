# Proposal: Admit the exact wrpc telemetry emitter artifact profile

## Summary

Add a `wrpc-telemetry-reference` Wasm artifact profile that admits the
normalized wasi-otel telemetry emitter Mantle materializes. It allows and
requires exactly the emitter's `wasi:otel/tracing` effect import and its
`wasi:otel/types` and `wasi:clocks/wall-clock` instance imports, and admits no
other import or proposal.

## Motivation

Mantle's `materialize-wrpc-telemetry` bundle stops at its Octet stage with
`octet-report-not-admitted`. Under `wrpc-reference` the normalized emitter is
denied with `disallowed-import` for
`component::wasi:clocks/wall-clock@0.2.3::Instance(1)`,
`component::wasi:otel/tracing@0.2.0-rc.2::Instance(5)` and
`component::wasi:otel/types@0.2.0-rc.2::Instance(0)`, and with
`resource-limit` (`imports count 3 exceeds profile limit 2`). The denial is
correct: no shipped profile admits those imports.

## Scope

- Add one additive profile to `standards/wasm-artifact/profiles.ncl` and
  regenerate `standards/wasm-artifact/generated/profiles.json`.
- Allow and require exactly the three observed import identities, with the
  `wasm1`, `component-model`, `bulk-memory` and `call-indirect-overlong`
  proposals the compiled emitter uses, three import slots and two tables. Every
  other profile stays closed against the emitter.
- Add the normalized emitter as a positive fixture, and Rust tests for its
  admission and for its denial under `wrpc-reference` and the portable
  baseline, plus the refusal of the wrpc hello server under the new profile.
- Record the export, focused-test and evidence-collect transcripts.

## Affected specs

- `wasm-component-artifact-rail`: exact wrpc telemetry emitter admission.

## Non-goals

- No admission of other WASI or otel imports, other versions, or other instance
  type indices.
- No claim that the types and wall-clock imports are type-only. Octet records an
  import's identity and instance type index, not the instance's contents; Mantle
  observes type-only use before Octet runs.
- No runtime, telemetry-export, clock, authority, provenance, or release claim.
