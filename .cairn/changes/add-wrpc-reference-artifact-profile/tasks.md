# Tasks

## Implementation

- [x] [serial] I1 Add the `wrpc-reference` profile to `standards/wasm-artifact/profiles.ncl`. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] I2 Regenerate `standards/wasm-artifact/generated/profiles.json` with pinned Nickel. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] I3 Add the normalized wrpc hello component fixture with provenance. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] I4 Add the positive-admission and baseline-denial Rust tests. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] I5 Document the profile in `standards/wasm-artifact/README.md` and `CHANGELOG.md`. r[octet.wasm_component.wrpc_reference_profile]

## Verification

- [x] [serial] V1 `cargo test -p cargo-octet --lib wrpc_reference` passes. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] V2 The exported JSON regenerates byte-identically from the Nickel source. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] V3 A direct `cargo octet evidence collect` over the fixture with the new profile returns `passed`; the portable baseline returns the two findings. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] V4 Focused formatting and Clippy pass. r[octet.wasm_component.wrpc_reference_profile]
- [x] [serial] V5 Record the transcripts under `evidence/` and retain explicit non-claims. r[octet.wasm_component.wrpc_reference_profile]
