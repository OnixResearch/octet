# no_todo expansion regression

Implementation: `ca08a7c73c27f07b11c5f0580d3d033b696bf7a0`.
Base: `fc38f59330b626961d166febfdf1a5aa6575460f`.

## Change and scope

`src/safety/no_todo.rs` no longer requires an expanded block before it examines a macro callsite.
A bare `todo!()` expands to a call on the existing March-21 compiler.
A message-bearing invocation introduces a block.
The old block condition therefore missed the bare form.

The pass now remembers each recognized source callsite.
This prevents duplicate diagnostics from nested expressions in one expansion.
Distinct invocations remain independent.
The source-snippet classifier, lint levels, test-context policy, and documented-allow policy remain unchanged.
This fix does not claim complete macro resolution or general lint soundness.

## Evidence

| Case | Result |
| --- | --- |
| Original four UI fixtures with the old library | Passed before the implementation change. |
| Extended fixtures with the candidate library, without `BLESS` | All four snapshots passed. |
| Same extended fixtures with the old library | Failed: two diagnostics instead of seven in `no_todo`. |
| Pure helper tests | Two passed. |
| Library Clippy and changed Rust files under pinned rustfmt | Passed. Clippy used `-D warnings`. |

The expanded detection snapshot contains seven diagnostics.
It covers message, bare-tail, bare-statement, braced, formatted, and two distinct branch callsites.
Neither test-only function form nor either `cfg(test)` form produces a diagnostic.
Ordinary string text does not produce a diagnostic.

The allow snapshots preserve justified function and block allowances.
They reject an undocumented bare stub and a documented allowance around `panic!()`.
A `cfg(not(test))` production stub produces a diagnostic.
A legacy `no_panic` allowance does not suppress a bare `todo!()`.
The existing orphan-allow snapshot also passes unchanged.

The strict smoke used the old pinned CLI with the new library deliberately.
The unchanged empty fixture exited zero.
The unchanged bare-todo fixture exited two with two error-level `no_todo` findings, one per target.
All workspace, target, and feature flags remained enabled through the pinned deny-all hook.
This mixed-component diagnostic experiment is not a new approved tool cohort or Molten startup receipt.

## Build identity

The final library build used the immutable implementation revision, offline, without substitutes or remote builders.
The existing compiler and driver remained unchanged.
Each audited build plan required only a library or test derivation.
No compiler, driver, Verus toolchain, Mantle, Darkhttpd, or package toolchain rebuild occurred.

- Complete source: `/nix/store/21zjbfa41h4j7qqnrgj2sdyc6rmq3w6a-source`.
- Complete source NAR: `sha256-hHIGUJ69HJWKBue0hlTC9sLKb7OQIDlEL884BgGbyTA=`.
- Filtered source: `/nix/store/10j9h8gvfqs0qcrqv39a4b1qqiv6fqrm-source`.
- Filtered source NAR: `sha256-CnXR/k/0kzSZI3F5rsCQeNnU9VisvXpIPc849LDgv2w=`.

Final derivation: `/nix/store/25rbm3gfg7ss8h5j2nnpcw13k2dh4rvl-octet-0.1.0.drv`.
Final output: `/nix/store/1wmd44w5qrpazl97g8rv0dfkaf3wzb9j-octet-0.1.0/lib/liboctet.so`.
Library BLAKE3: `10919bdd10b0049c1b1113f9757dc563441c6709b91efb02323ff5d32489447b`.

The development library at `/nix/store/pfy0qmxclc6cnj24f5vwglc2fgbgiay2-octet-0.1.0/lib/liboctet.so` supplied the UI and smoke runs.
A byte comparison and BLAKE3 measurement matched it to the final library.
This byte equality does not equate the two source inventories.
The final build independently records its complete immutable input.
Nix `store verify --no-trust` passed for the final output. This is a content check, not signature verification.

## Replay

From the implementation checkout, run the saved helper with existing tool paths:

```sh
nix --extra-experimental-features 'nix-command flakes' build \
  --offline --no-substitute --no-link --dry-run \
  --file verification/no-todo-expansion/check.nix \
  --arg repository ./. \
  --arg library /nix/store/pfy0qmxclc6cnj24f5vwglc2fgbgiay2-octet-0.1.0 \
  --arg driver /nix/store/v685hxclxij36n3bcgal3xpv64q4bkqv-dylint-driver-5.0.0 \
  --arg runtime /nix/store/arsdclqr5dk7qd51ap89p50mvq4pc8bz-cargo-octet-runtime-path
```

Inspect the plan before execution. Stop if it requires an excluded toolchain or a driver build.
Replace `--dry-run` with `-L --max-jobs 1 --cores 2 --timeout 600 --max-silent-time 180 --builders ''`.
The saved helper always removes `BLESS` from acceptance runs.
The recorded successful helper replay built `/nix/store/4zgbwyprm1pzib4n9vzynirzc4ygyrlg-octet-no-todo-ui-0.1.0.drv`.
Its output is `/nix/store/ffrc69xn67wa7jf828as8xhvx5sms18b-octet-no-todo-ui-0.1.0`.

For the negative replay, select the old library at `/nix/store/d5kynl0p00wailfm1swkm3rbc7bm7hg8-octet-0.1.0`.
The expected result is a snapshot mismatch, not success.
Mode `quality` runs the two helper tests and library Clippy.
Mode `all-ui` runs the broader ignored UI suite.

## Remaining limits

The broader UI suite fails on both the baseline and candidate.
The `unbounded_channel` fixture lock requires `crossbeam-channel 0.5.15`, but the pinned vendor directory supplies `0.5.16`.
Both runs reached that same dependency failure. Their focused regression and autofix tests passed.
No fixture lock, vendor input, or expected failure was changed to hide this blocker.
Full workspace, full Octet gate, and flake acceptance remain unproved.

The first private harness used an ineffective `cargoTestExtraArgs` override and ran the broader suite unintentionally.
The corrected helper overrides `checkPhase` explicitly. The first failure remains in the evidence.
Snapshot generation used `BLESS` once. The later acceptance and old-library negative runs did not use it.

Molten stays at `c9cd279ce62eaf4e19a81b59e64c8eea3c5386b1`.
Its Octet pin, manifests, policy, and startup guards remain unchanged.
The Onix consumer stays at `cbb77ccfe5d4a4f6ac8ae25b5fdc4a1540adc54c` with its VM guard unchanged.
No normal Molten gate, VM, native replay, Stage0, deployment, or release promotion occurred.
The implementation patch applies cleanly to the inspected Octet main revision `cf04e894e53eb0947230118a086ef6066ddba38c`.
That applicability check does not prove compatibility or perform a merge.
Review was single-agent and correlated.
