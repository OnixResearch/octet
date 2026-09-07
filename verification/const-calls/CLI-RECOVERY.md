# Check-only CLI recovery

The original CLI/runtime store outputs were missing. They were not recovered exactly.
Commit `21379f9` adds `check-runtime.nix` for diagnostic checks with the existing March-21 compiler.
Standard packaging remains unchanged. The diagnostic runtime does not supply Verus or establish formal-proof support.

## Audited build

Task 10340 inspected the standard runtime plan. It required 17 derivations, including Rust 1.94 and production Verus acquisition and packaging.
That plan was rejected without realization.
Task 10342 accepted only the unwrapped CLI package, Cargo shim, and check-runtime join.
Task 10343 realized those three derivations in 97.597 seconds with no substitutes, remote builders, or network acquisition.
The existing compiler, dependency artifacts, driver, and lint library were reused.
No compiler, Verus, Mantle, Darkhttpd, or Stage0 build ran.

The CLI source is `f42bc855d496db43cee49dbe7198f98f2bdd09ef`.
Build output includes the package's `cargo-octet`, `cargo-tigerstyle`, and `cargo-slotcar` executables.
The build emitted 15 dead-code warnings. It did not run CLI unit tests.

- CLI derivation: `/nix/store/ksla5mcjcwjjfgzpryadk5n40c2izi01-cargo-octet-0.1.0.drv`
- CLI output: `/nix/store/vr0w4sf0l19rgjk3xgxk44zzjkd7sg4w-cargo-octet-0.1.0`
- CLI executable BLAKE3: `d5d23275c81c9c5a157515fc3ca6f32e79b043d02ddf6185bc113c3b9f37c066`
- Runtime derivation: `/nix/store/pab4m28w5dq1h1zxgz7cck5vfj9fbb85-cargo-octet-check-runtime.drv`
- Runtime output: `/nix/store/4jfn77sinrbpda0gy4amrnjmyjaz3nl4-cargo-octet-check-runtime`

Task 10352 retained both outputs with private GC roots and passed content verification with `nix store verify --no-trust`.
This is not signature or reproducibility evidence.

## Replay and scope

Use the existing `verification/driver-dependency-flags/verify.sh` with absolute arguments:
`OUT DRIVER CLI RUNTIME COMPILER LIBRARY HOOK`. `OUT` must be fresh.
Task 10347 passed its three workspace selections and six direct-driver controls with the replacement tools.
The supplied library was the retained `40f34...` const-call output, not a rebuilt library.

Molten task 10351 then ran the unchanged canonical command on frozen source `ab3efbe9788643caddd7f22bb0a13eca0d90426f`.
It reported 33 node-host errors, down from 36, with zero warnings.
Only the three const findings disappeared. This is not complete workspace coverage or startup approval.
No normal-node VM launched.

Private full logs and conformance outputs live under `~/.local/state/onix/molten-node-vm/cli-recovery/`.
The producer change retains digests and source-gate observations at `.cairn/changes/node-startup-evidence/evidence/cli-recovery/`.
Initial inspection failures 10336 and 10337 remain retained; neither started a build.
