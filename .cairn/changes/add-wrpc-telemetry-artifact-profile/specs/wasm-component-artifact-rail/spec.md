# wasm-component-artifact-rail delta

## ADDED Requirements

### Requirement: the wrpc telemetry emitter has an exact profile

r[octet.wasm_component.wrpc_telemetry_profile] Octet MUST define a recorded-only `wrpc-telemetry-reference` profile that admits exactly the normalized wasi-otel telemetry emitter: it MUST allow and require the instance imports `component::wasi:clocks/wall-clock@0.2.3::Instance(1)`, `component::wasi:otel/tracing@0.2.0-rc.2::Instance(5)` and `component::wasi:otel/types@0.2.0-rc.2::Instance(0)`, allow only the `wasm1`, `component-model`, `bulk-memory` and `call-indirect-overlong` proposals, three import slots and two tables, and admit no other import, while `wrpc-reference` and the portable baseline remain closed against the same artifact.

#### Scenario: The normalized emitter is admitted
- GIVEN the normalized telemetry emitter and the `wrpc-telemetry-reference` profile
- WHEN artifact evidence is collected
- THEN Octet MUST return no findings with recorded-only verification
- AND the `wrpc-reference` and `portable-component-baseline` profiles MUST still deny the same bytes.

#### Scenario: A component without the required imports is refused
- GIVEN the wrpc hello server component evaluated under `wrpc-telemetry-reference`
- WHEN artifact evidence is collected
- THEN Octet MUST fail closed with an `abi-mismatch` finding for each missing required import and no runtime claim.
