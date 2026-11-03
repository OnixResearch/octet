# Design: wrpc reference artifact profile

## Profile

`wrpc-reference` is a recorded-only component profile:

- `allowed_artifact_kinds = ["component"]`
- `allowed_proposals = ["wasm1", "component-model", "bulk-memory",
  "call-indirect-overlong"]`
- `allowed_imports = ["component@3::import-func-hello::Func(0)", "component::wrpc-examples:hello/handler::Instance(0)"]`
- `allow_any_exports = true`, `required_exports = []`, `max_imports = 2`, `max_tables = 2`
- the same resource maxima, severity map, metadata policy, and non-claims as the
  portable baseline, plus a wrpc-specific non-claim.

The server import identity includes the parser scope ordinal; the client imports
the hello handler as a component instance. Both identities are exact, so the
profile admits exactly the observed pair and not a differently scoped import.

## Evidence

- `standards/wasm-artifact/generated/profiles.json` is regenerated with pinned
  Nickel 1.17.0 and must match the exported source.
- `standards/wasm-artifact/fixtures/positive/wrpc-hello-component.wasm` and
  `wrpc-hello-client-component.wasm` are the normalized server and client
  artifacts from the wrpc reference bundle at revision
  `aa01409ed38bf7a978570ff567c4475376925df6`; provenance is recorded in the
  fixture note.
- `cargo-octet` test `wrpc_reference_profile_admits_the_exact_hello_component_and_keeps_the_baseline_closed`
  proves admission and the portable-baseline denial.

## Failure behavior

An unknown import, another proposal, an oversized artifact, or a different
component fails closed through the existing profile validator with the ordered
findings and unchanged severity map.
