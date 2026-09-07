# Dylint dependency plugin flags

## Repair

Implementation commit: `b7abcb24eb36afb20d62c037ab35daffb25d23d8`, based on startup Octet pin `c9b06bcf565c51d4a77d210e61b69ae51db9df25`.

The driver previously skipped dependency lint registration under `DYLINT_NO_DEPS=1`, but still appended `DYLINT_RUSTFLAGS`.
The all-deny hook then failed on unregistered plugin names in non-primary workspace dependencies.

One pure predicate now controls both registration and plugin-flag selection.
Primary packages retain their plugin flags, including unknown-lint denial.
With no-deps disabled or absent, non-primary packages retain plugin flags and lint registration.
Ordinary compiler arguments remain unchanged.

`patches/dylint-dependency-flags.patch` applies to the pinned upstream driver source.
The source derivation compares the patched file with `tests/support/dylint/driver/src/lib.rs` before building.
This keeps the packaged driver and checked-in test-support owner synchronized.
No library lint levels, CLI scope, hook flags, or Molten policy were changed.

## Bounded build

The user approved this driver-only repair/build after the six-probe reproduction.
The dry run and build scheduled exactly two derivations:

- `xnv8x25vicqk45zn0szmqg199j5sk7wf-dylint-driver-src.drv`
- `03i22rcwhwrbm2gyp9r3mlnz6h976fg0-dylint-driver-5.0.0.drv`

Task 10735 completed with exit 0 in 13.270 seconds.
Nix used offline mode, no substitutes, no remote builders, one build job, two cores, and bounded timeouts.
The systemd counters describe the Nix client; they do not measure daemon compiler work.
No compiler, CLI, lint library, Verus, Mantle, Darkhttpd, or package-toolchain build was scheduled.
The existing March-21 compiler was reused.

The driver build runs `cargo test --release --offline --locked --lib`.
All six tests passed, including scope-matrix and ordinary-argument preservation tests.

Output:

```text
/nix/store/r5bzbvda2ydnz09c6vxvqhxsmh37nhpw-dylint-driver-5.0.0/bin/dylint-driver
BLAKE3 8e14cfcb3f0cfd993c5886e574408e11a49456dbb637b766e3d512e698d4474b
```

Packaged source:

```text
/nix/store/2za6b6maandmwvdclamf9qsicqjyf0dk-dylint-driver-src/driver/src/lib.rs
BLAKE3 abd0c5a7ab4219a02cd747c6b6bfdd432f5bc924862c685f743985a81496c101
```

The packaged file matches the checked-in file. This does not equate whole source trees.
Nix store content verification passed with `--no-trust`; no signature or reproducibility claim follows.

## Behavioral checks and replay

Run with absolute paths and a fresh output directory:

```sh
sh verification/driver-dependency-flags/verify.sh \
  OUT DRIVER CLI_PACKAGE RUNTIME_PACKAGE COMPILER_PACKAGE LIBRARY_FILE HOOK_FILE
```

The helper never builds tools. It uses existing tools, fresh target directories, and an isolated offline environment.
The workspace fixture was reused from Molten commit `bb6abd614ca0fe669c18932ce77b28d566bb46fe`, under owner authorization.
The empty libraries and bare-todo input are diagnostic fixtures, not production source evidence.

Task 10742 passed all nine behavioral checks:

1. App primary with an empty workspace dependency: clean, zero findings.
2. Both workspace crates primary: clean, zero findings.
3. Core primary alone: clean, zero findings.
4. Direct empty non-primary package with no-deps enabled: exit 0.
5. Direct empty primary package: exit 0.
6. Primary bare `todo!()`: exit 101 with the actual no-todo diagnostic.
7. Non-primary bare `todo!()` with no-deps disabled: exit 101 with the actual diagnostic.
8. Primary unknown lint: exit 101 with an unknown-lint diagnostic.
9. Primary missing library: exit 101 with a library-load diagnostic.

The helper failed against the old driver in task 10743: exit 2, 68 unknown-lint errors, one warning.
That negative result establishes that the new helper detects the original defect.
The driver unit tests and behavioral checks do not establish full Octet workspace/UI/flake acceptance.
The existing unrelated UI vendor mismatch remains outside this repair.

## Retained setup failures

The initial formatter command referenced a nonexistent upstream rustfmt config and failed before changing files.
The first staged diff check flagged normal blank context lines inside the patch; patch-specific whitespace attributes now cover those lines.
The first conformance unit used a relative helper path and exited 127 before any probe. The successful run used an absolute path.
These attempts are retained, not counted as passing checks.

## Consumer boundary

Molten's producer pin and startup guards are not changed by this driver repair.
The conformance check combines the repaired driver with the previously measured CLI/library; it is not an approved full-source runtime cohort.
A separate diagnostic run uses Molten's unchanged command and exact frozen source from commit `6667e5616b978f7bb6f64201b76d452634815612`.
Task 10745 completed that diagnostic in 1 minute 46.382 seconds, with exit 2 and Cargo exit 101.
The source archive is byte-identical to the original failed attempt. Tool/input identities match before and after; the tracked-source diff is empty.
The registration error is gone. Octet now reports 106 errors in `molten-node-host`, including 64 non-trait imports and 18 repeated path-segment names.
This is not a clean Molten gate or complete workspace coverage. Consumer-side evidence retains the new failure separately from the original 68 unknown-lint errors.
No Stage0, production-runtime build, VM, native replay, physical deployment, main integration, or startup authority is claimed.
