# Const declaration and call eligibility

Implementation: `1e7b64f9c1a56052a51dc98e7538cbcb8531d15a`, based on `706b47d6957835fbe46213d5573bebfe3630a66e`.

## Repair

`MissingConstFn` now excludes trait declarations and trait-implementation methods through compiler definition queries.
Those methods cannot acquire an individual `const` qualifier.
A new HIR visitor also excludes resolved direct and method calls whose function definitions are not const.
Constructor definitions are not treated as ordinary functions. Closure bodies remain separate scopes.
The existing runtime-format exclusion, source classifier, test-context rules, diagnostic levels, and registration remain unchanged.

This is a negative eligibility check, not a complete const proof.
It does not establish const-feature availability, conditional trait bounds, destructor legality, indirect-call eligibility, or transitive eligibility.
In particular, `is_const_fn` does not establish stability. These limitations must not become claims of general const safety.
The known prefix-helper failures include genuinely non-const string calls, so they are excluded without guessing feature availability.
No source-text blacklist or suppression was added for these cases.

## Verification

The audited offline plan contained one lint-library derivation, with no substitutes or remote builders.
Task 10318 built it with one job, two cores, and bounded execution. Seven existing owner tests passed.
The build took 24.422 seconds. Systemd resource counters cover the Nix client, not daemon compiler work.
No compiler, repaired driver, CLI, Verus, Mantle, Darkhttpd, or package toolchain was rebuilt.

```text
Derivation: /nix/store/h5d4z9iqxjfw0dzwp84igba8ybj5qqpq-octet-0.1.0.drv
Library: /nix/store/40f34yq9qpxmf4k3cdlcj24fj14gjh9f-octet-0.1.0/lib/liboctet.so
BLAKE3: 03e24c4900d0f403de955bd30786b17847e7aef9377694ea81fb27ba23e510e2
```

The output uses `verification/const-allocation/check.nix`, not the default package derivation.
The existing March-21 compiler and repaired driver were reused.
Nix content verification passed with `--no-trust`; a private GC root retains the library.
These facts do not establish signatures, reproducibility, or an approved runtime cohort.

Task 10319 passed ten retained controls plus nine new paired controls.
The new controls first compile normally, then use the driver with unknown-lint denial and missing-const denial.
They cover the exact Molten Display implementation and both prefix helpers, plus direct non-const calls, method calls, and trait defaults/implementations.
Private inherent methods, calls to const methods, and enum construction retain valid suggestions.
The earlier formatting, unnamed-impl-parent, positive-candidate, and bare-todo controls still pass.
The helper rejects compiler crashes.
Task 10321 runs the same helper with the preceding library and fails on the Display case, after the retained controls pass.

The three Molten fixtures come from producer commit `5f92e2dc9201eae6f1271d0dcbdacd805f5e268a`, under its const-review evidence directory.
They are regression inputs, not a new product dependency.
The compiler rejection evidence there remains valid; this repair removes incorrect suggestions rather than changing those functions.

## Replay

Use absolute arguments and a fresh output directory:

```sh
sh verification/const-calls/verify.sh OUT DRIVER LIBRARY COMPILER
```

The helper builds no tools or runtime. It starts no service.
Full Octet UI/workspace/flake acceptance, full Clippy, and main integration are not established here.
The existing UI vendor blocker remains outside this repair.

## VM boundary

This repair supports the requested VM-only path; it is not permission to bypass startup admission.
Molten must still pass its unchanged complete-source gate and bind an approved cohort to the runtime before normal-node VM deployment.
No Molten pin, policy, scope, hook, startup guard, production build, VM, physical device, or release state changed here.

The attempted frozen Molten run (task 10323, lifecycle-source-gate-11) stopped before invoking the CLI.
The retained c14wxbcz CLI and pp4l3in8 hook store paths are missing. The arsdclqr runtime wrapper path is also absent.
The source archive matched run 10, but the preflight identity command failed. No new source-gate status was produced.
Nine locally discovered CLI executables have different digests from the retained CLI; none was substituted.
A hook recovered from pinned source has the expected digest, but it does not restore the missing CLI or runtime wrapper.
The last completed full command still reports 36 findings. The successful direct controls do not establish a reduced full-gate count.
Next: restore the exact CLI/runtime tools or explicitly audit a replacement tool build before the next complete-source diagnostic.
