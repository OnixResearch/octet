# Design: wrpc telemetry emitter artifact profile

## Profile

`wrpc-telemetry-reference` is a recorded-only component profile:

- `allowed_artifact_kinds = ["component"]`
- `allowed_proposals = ["wasm1", "component-model", "bulk-memory", "call-indirect-overlong"]`
- `allowed_imports` and `required_imports` both hold exactly
  `component::wasi:clocks/wall-clock@0.2.3::Instance(1)`,
  `component::wasi:otel/tracing@0.2.0-rc.2::Instance(5)` and
  `component::wasi:otel/types@0.2.0-rc.2::Instance(0)`
- `allow_any_exports = true`, `required_exports = []`, `max_imports = 3`, `max_tables = 2`
- the portable baseline's remaining resource maxima, severity map and metadata
  policy, with the baseline non-claims plus a telemetry-specific one.

Each identity carries the instance type index the parser observed, so the
profile admits exactly the observed emitter and not a differently scoped
import. Requiring the three imports as well as allowing them keeps the
profile from admitting a component that drops the tracing effect.

The emitter is Rust wasm32 output: it uses bulk memory and the overlong
`call_indirect` table encoding, so the profile allows the same proposals as
`wrpc-reference`. Without them wasmparser validation fails, no core
instantiation counts as bound, and the internal shim and fixup imports of the
indirect-lowering pattern surface as external imports. The two tables are the
main module's indirect-call table and the shim's `$imports` table.

## Evidence

- `standards/wasm-artifact/generated/profiles.json` is regenerated with the
  pinned Nickel and must match the exported source.
- `standards/wasm-artifact/fixtures/positive/wrpc-telemetry-emitter-component.wasm`
  is the pipeline's final normalized emitter (`artifacts/normalized.wasm`,
  25,698 bytes, BLAKE3
  `0a701cfe9279b6e72bbcace7a1987c5030cfb1e7f25844a55b14dd51efd84f6f`). Mantle
  `994bf6537` produced it from wasi-otel `e5262a84044aff064c9599eebc05013bb3432795`
  through `mantle-wrpc-author telemetry` and `mantle wasm-component build`;
  that run stopped at its Octet stage with the denial quoted in the proposal.
- `cargo-octet` tests prove admission under the new profile and the denials
  under `wrpc-reference` and `portable-component-baseline`, and refuse the wrpc
  hello server under the new profile.

## Failure behavior

Another import, a missing required import, another proposal, an oversized
artifact, or a different component fails closed through the existing profile
validator with ordered findings and the unchanged severity map.
