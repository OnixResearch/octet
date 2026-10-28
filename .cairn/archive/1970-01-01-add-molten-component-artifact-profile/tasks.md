# Tasks: Add a Molten component artifact profile

- [x] [serial] Add the `molten-component-baseline` Nickel profile and regenerate the projection. r[octet.molten_profile.profile]
- [x] [serial] Recognize the granular `bulk-memory` proposal and map it to the bulk-memory feature only. r[octet.molten_profile.granular_proposal]
- [x] [serial] Keep `portable-component-baseline` unchanged. r[octet.molten_profile.baseline_unchanged]
- [x] [serial] Admit only `call-indirect-overlong` for the reviewed LLVM-linked zero table immediate; retain reference-type and SIMD denials. r[octet.molten_profile.indirect_encoding]
- [x] [serial] Run the focused wasm-artifact tests and exact supported-world artifact smoke. r[octet.molten_profile.validation]
  - Executed 15 library tests, strict package Clippy, the focused Nix rail and the real guest positive/negative smoke. See `evidence/2026-10-28-indirect-encoding.md`. Package-wide formatting remains non-blocking only for unrelated existing `tests/fingerprint_consumer.rs` formatting; the changed source passes its exact-file check.
