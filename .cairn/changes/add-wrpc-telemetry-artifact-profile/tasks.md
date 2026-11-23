# Tasks

## Implementation

- [x] [serial] I1 Add the `wrpc-telemetry-reference` profile to `standards/wasm-artifact/profiles.ncl`. r[octet.wasm_component.wrpc_telemetry_profile]
- [x] [serial] I2 Regenerate `standards/wasm-artifact/generated/profiles.json` with pinned Nickel. r[octet.wasm_component.wrpc_telemetry_profile]
- [x] [serial] I3 Add the normalized telemetry emitter fixture with provenance. r[octet.wasm_component.wrpc_telemetry_profile]
- [x] [serial] I4 Add the positive-admission and denial Rust tests. r[octet.wasm_component.wrpc_telemetry_profile]
- [x] [serial] I5 Document the profile in `standards/wasm-artifact/README.md` and `CHANGELOG.md`. r[octet.wasm_component.wrpc_telemetry_profile]

## Verification

- [x] [serial] V1 `cargo test -p cargo-octet --lib wrpc_` passes. r[octet.wasm_component.wrpc_telemetry_profile]
  - [focused tests](evidence/2026-11-18-focused-tests.txt): 3 passed.
- [x] [serial] V2 The exported JSON regenerates byte-identically from the Nickel source. r[octet.wasm_component.wrpc_telemetry_profile]
  - [Nickel export](evidence/2026-11-18-nickel-export.txt): Nickel 1.17.0 typecheck and byte comparison exit 0.
- [x] [serial] V3 A direct `cargo-octet evidence collect` over the fixture with the new profile returns `passed`; `wrpc-reference` and the portable baseline return their denials. r[octet.wasm_component.wrpc_telemetry_profile]
  - [artifact evidence](evidence/2026-11-18-artifact-evidence.txt).
- [x] [serial] V4 Focused formatting and Clippy pass. r[octet.wasm_component.wrpc_telemetry_profile]
  - [formatting and Clippy](evidence/2026-11-18-format-clippy.txt).
- [x] [serial] V5 Record the transcripts under `evidence/` and retain explicit non-claims. r[octet.wasm_component.wrpc_telemetry_profile]
