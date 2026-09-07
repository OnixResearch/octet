# Preserve vector parameters that require growth

Implementation: `9e0e455`, based on `faf6ed4b210ebb7b514bbd53f0a23adf1139df98`.
Owner: `src/structure/borrowed_argument_types.rs`.

## Diagnosis and repair

Molten's bounded append helpers call `Vec::push` after capacity admission.
The existing lint suggested mutable slices, which cannot preserve those operations.
Task 10391 confirmed that the vector fixture compiles, the slice fixture fails with E0599, and the old library reports the invalid suggestion.
The entry fixture uses `u64` instead of Molten's entry type and a standalone capacity guard; it is not an exact full-function copy.

The repair recognizes resolved inherent `alloc::Vec::push` calls on the exact parameter binding.
It supports method syntax and direct associated-function syntax, for free functions and inherent methods.
The resolved implementation's ADT must match the candidate vector definition.
Unrelated local vectors and shadowed names do not exempt a parameter.
Other parameters remain checked after a required vector parameter is skipped.

This is a narrow required-operation check, not a general ownership analysis.
It does not track aliases, reborrow expressions, captured closure uses, destructured bindings, arbitrary helper calls, or other container-specific operations.
Those unsupported cases retain existing diagnostic behavior.
Mutable vectors are not exempted wholesale. Trait-implementation and extern handling, diagnostic levels, scope, and suggestions remain unchanged.
No Molten signature, guard, effect order, API name, or policy changed.

## Verification

Task 10391 audited one lint-library derivation.
Task 10392 built it in 24.537 seconds with the existing March-21 compiler and dependency artifacts.
Seven existing purity-classifier unit tests passed; these are regression tests, not new borrowed-interface unit tests.
No compiler, driver, CLI, runtime, Verus, Mantle, Darkhttpd, or Stage0 build ran.

Task 10394 passed:

- Four new compiler/driver probes: vector compilation, slice rejection, growth exemption, and seven retained candidates.
- Growth coverage: both bounded helper shapes, direct `Vec::push`, and an inherent method with `self`.
- Candidate coverage: shared vector, mutable slice-compatible use, another local vector, shadowed binding, a second read-only parameter, String, and PathBuf.
- All 36 prior probes/controls: eight arithmetic, 19 const, and nine workspace/driver checks.

Bare-todo, unknown-lint, missing-library, runtime-arithmetic, and invalid-constant rejection remain covered.
No full Octet UI/workspace/flake/Clippy acceptance is claimed.

Exact new-control replay from this worktree:

```sh
sh verification/borrowed-growth/verify.sh \
  /home/brittonr/.local/state/onix/molten-node-vm/borrowed-growth/conformance \
  /nix/store/r5bzbvda2ydnz09c6vxvqhxsmh37nhpw-dylint-driver-5.0.0/bin/dylint-driver \
  /nix/store/r5aw3lz2sa6h2cg9lphy881w0w1ra2id-octet-0.1.0/lib/liboctet.so \
  /nix/store/1yvh3d6y3fj3xk2dgwczrp1dj5svd92c-rust-default-1.96.0-nightly-2026-03-21
```

That output now exists; choose a fresh absolute output for another replay.
The helper asserts exit codes, exact candidate count, no growth diagnostics, E0599 rejection, absence of ICE markers, and equal before/after identities.

## Complete-command diagnostic

Molten task 10397 ran the unchanged canonical command on frozen `ab3efbe9788643caddd7f22bb0a13eca0d90426f`, with only the library changed from run 13.
It reported 30 node-host errors, down from 32, with zero warnings; exit 2/Cargo 101.
Only the two borrowed-interface findings disappeared. Recorded identities matched, source stayed unchanged, and no ICE markers appeared.
This is not complete workspace coverage. Startup remains unauthorized; no normal-node VM launched.

## Retained tool

- Library: `/nix/store/r5aw3lz2sa6h2cg9lphy881w0w1ra2id-octet-0.1.0/lib/liboctet.so`
- BLAKE3: `e465f12461fb498e21a531d2c48a36c6fb4338abd60429f6cfba7622ecc9d194`
- Derivation: `/nix/store/1j1vzz58d13h485jf9g6wlk6mgv2gnjy-octet-0.1.0.drv`
- Build entry: `verification/const-allocation/check.nix` with this repository, not the default package output.
- Private evidence: `~/.local/state/onix/molten-node-vm/borrowed-growth/`.

A private GC root retains the output; local content verification with `nix store verify --no-trust` passed.
This is not signature, reproducibility, formal-proof, runtime-cohort, or startup-authorization evidence.
