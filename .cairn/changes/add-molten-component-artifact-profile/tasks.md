# Tasks: Add a Molten component artifact profile

- [x] [serial] Add the `molten-component-baseline` Nickel profile and regenerate the projection. r[octet.molten_profile.profile]
- [x] [serial] Recognize the granular `bulk-memory` proposal and map it to the bulk-memory feature only. r[octet.molten_profile.granular_proposal]
- [x] [serial] Keep `portable-component-baseline` unchanged. r[octet.molten_profile.baseline_unchanged]
- [ ] [serial] Run the focused wasm-artifact tests. r[octet.molten_profile.validation]
  - Blocked in this environment: `cargo test -p cargo-octet` cannot resolve the private `bounded-exec` git dependency (`44eb74eb… not found`) and the repo flake requires an untracked `nix/public-dependencies.nix`. The positive/negative test is added; execution awaits dependency acquisition.
