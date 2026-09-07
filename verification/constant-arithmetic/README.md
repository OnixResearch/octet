# Bounded literal-arithmetic eligibility

Implementation: `743f5be`, based on `8e7bc5d8bd77ee7ef461452745fcb78132461df1`.
Owner: `src/safety/raw_arithmetic_overflow.rs`.

## Repair boundary

The existing constant exemption recognized only literal operands.
It therefore diagnosed the outer multiplication in `16 * 1_024 * 1_024`, even in a constant initializer.
The compiler accepts this expression and rejects genuine constant overflow.

The predicate now also recognizes literal-only addition, subtraction, and multiplication trees.
Recursion has a 32-level budget per inspected operand; deeper expressions remain lint candidates.
Budget subtraction is checked. Variables, named constants, casts, calls, and other unsupported forms remain candidates.
The repair does not exempt a body merely because it belongs to a `const fn`.
It is a bounded syntax recognizer, not a general constant evaluator or overflow proof.
Diagnostic levels, registration, scope, and runtime suggestions remain unchanged.

## Verification

Task 10364 audited a single lint-library derivation.
Task 10365 built it using the existing March-21 compiler and dependency artifacts in 23.927 seconds.
It passed seven existing purity-classifier unit tests. These are regression tests, not new arithmetic unit tests.
No compiler, driver, CLI, runtime, Verus, Mantle, Darkhttpd, or Stage0 build ran.

Task 10366 passed eight arithmetic probes:

1. Literal arithmetic compiles and produces no Dylint diagnostics.
2. Checked operations produce no diagnostics.
3. All five unchecked runtime operations remain diagnosed, including a parameter-dependent `const fn` and mixed literal/variable multiplication.
4. An expression beyond the recursion budget compiles but remains a lint candidate.
5. Both rustc and Dylint reject true nested constant overflow with E0080.

The same task passed the ten const-format controls and nine const-call controls.
Task 10368 passed three workspace selections and six direct-driver controls, retaining bare-todo, unknown-lint, and missing-library denial.
It also confirmed the old library fails the corrected constant control.
No full Octet UI/workspace/flake/Clippy acceptance is claimed.

Exact arithmetic replay from this worktree:

```sh
sh verification/constant-arithmetic/verify.sh \
  /home/brittonr/.local/state/onix/molten-node-vm/constant-arithmetic/conformance \
  /nix/store/r5bzbvda2ydnz09c6vxvqhxsmh37nhpw-dylint-driver-5.0.0/bin/dylint-driver \
  /nix/store/a6zhf51jr25sgy0w4drm3ixabjilcgn3-octet-0.1.0/lib/liboctet.so \
  /nix/store/1yvh3d6y3fj3xk2dgwczrp1dj5svd92c-rust-default-1.96.0-nightly-2026-03-21
```

That output now exists; choose a fresh absolute output for another replay.
The helper asserts exit codes, diagnostic counts, compiler overflow rejection, absence of ICE markers, and equal before/after input identities.

## Complete-command diagnostic

Molten task 10371 ran the unchanged canonical command on frozen `ab3efbe9788643caddd7f22bb0a13eca0d90426f` with only the library changed from run 12.
It reported 32 node-host errors, down from 33, with zero warnings; exit 2/Cargo 101.
Only the constant-arithmetic finding disappeared. Recorded identities matched, source stayed unchanged, and no ICE markers appeared.
This is not complete workspace coverage. Startup remains unauthorized; no normal-node VM launched.

## Retained library

- Output: `/nix/store/a6zhf51jr25sgy0w4drm3ixabjilcgn3-octet-0.1.0/lib/liboctet.so`
- BLAKE3: `b4d36f3e3d6cb8c30cbf92b596cf9944d92ec2435f0c72c13f7d9db9b7c3965a`
- Derivation: `/nix/store/07c5gzsyairgk0mv61dxcpkjxr3p7rxr-octet-0.1.0.drv`
- Build entry: `verification/const-allocation/check.nix`, with this repository argument; not the default package output.
- Private evidence: `~/.local/state/onix/molten-node-vm/constant-arithmetic/`.

A private GC root retains the library. Local content verification with `nix store verify --no-trust` passed.
This is not signature, reproducibility, formal-proof, runtime-cohort, or startup-authorization evidence.
