# Proposal: Admit the exact wrpc reference artifact profile

## Summary

Add a `wrpc-reference` Wasm artifact profile that admits the normalized
`bytecodealliance/wrpc` reference hello component pair: the exact server host
import and client handler instance import, bulk memory and padded indirect
calls, and no other imports.

## Motivation

Mantle materializes the wrpc reference components as a signed producer/consumer
fixture. The bundle's Octet stage rejects the normalized artifact under
`portable-component-baseline` with `disallowed-import` for
`component@3::import-func-hello::Func(0)` and `disallowed-proposal` for bulk
memory. No shipped profile admits that identity, so the wrpc reference bundle
cannot produce admitted artifact evidence.

## Scope

- Add one additive profile to `standards/wasm-artifact/profiles.ncl` and
  regenerate `standards/wasm-artifact/generated/profiles.json`.
- Admit exactly the observed import identity and the bulk-memory /
  padded-indirect-call proposals; keep the portable baseline closed.
- Add the normalized artifact as a positive fixture and a Rust test that also
  proves the baseline denial.
- Record the deterministic export and focused test transcripts as evidence.

## Affected specs

- `wasm-component-artifact-rail`: exact wrpc reference profile admission and
  baseline closure.

## Non-goals

- No change to the wasm-tools cohort, resource maxima, or finding ordering.
- No admission of other wrpc imports, WASI, or reference types.
- No runtime, execution, authority, provenance, or release claim.
