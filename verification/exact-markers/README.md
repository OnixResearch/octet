# Exact policy-marker admission

Implementation: `da2f6a1`, based on `6b581a980e7c114304bf20743ad4ed705d97a113`.

## Owner and boundary

`src/safety/policy_markers.rs` now owns marker admission for both fragile enum matches and FSM core/enum markers.
It compares the complete configured path with compiler-parsed HIR attribute path segments through `Attribute::path_matches`.
Only custom tool attributes (`Attribute::Unparsed` in this compiler API) qualify. That variant has a parsed `AttrPath`; this is not source-text matching.
Empty path segments fail closed. Debug output, doc strings, suffixes, and unrelated payloads confer no marker status.

The retained compiler source documents the API in `compiler/rustc_hir/src/hir.rs`: `AttrPath`, `Attribute`, and `AttributeExt::path_matches`.
Both prior owners searched rendered attribute debug text for a name or tail segment. That let doc text simulate enum sealing or FSM opt-in/admission.
Neither owner now contains that marker-matching policy body; both delegate to the shared helper.

Ordinary unmarked-enum policy, configured path scopes, transition-matrix logic, diagnostic levels, and source-gate flags remain unchanged.
This does not declare any Molten domain sealed or authorize startup.

## Verification

Task 10430 audited one library derivation. Task 10432 built it in 26.554 seconds using the existing March-21 compiler and dependency artifacts.
Five existing FSM policy unit tests and seven existing purity-classifier tests passed.
No compiler, driver, CLI, runtime, Verus, Mantle, Darkhttpd, or Stage0 build ran.

Task 10433 passed 19 new probes:

- Genuine sealing and FSM enum markers accepted; unmarked enums still diagnosed.
- Doc text, wrong namespace, unrelated payload, name prefix, and extra path segment rejected.
- Custom configured namespace accepted; the former default rejected under that configuration; empty configuration fails closed.
- Adding an enum variant still produces E0004 despite a genuine marker.
- Genuine FSM core/enum markers accepted; doc and wrong-namespace enum markers rejected.
- Genuine FSM wildcard transitions rejected. Doc or wrong-namespace core names do not opt functions into the FSM profile.

The same task passed all 40 earlier growth/arithmetic/const/workspace/driver probes and controls.
This is not full Octet UI/workspace/flake/Clippy acceptance.

Configuration fixtures are Nickel-authored and exported to TOML for Dylint.
Task 10428 found no ambient Nickel and stopped before formatting or building.
An existing Nickel 1.17 binary was then selected explicitly; its identity is included in probe before/after digests.
No Nickel acquisition or build ran.

Exact replay from this worktree:

```sh
sh verification/exact-markers/verify.sh \
  /home/brittonr/.local/state/onix/molten-node-vm/exact-markers/conformance \
  /nix/store/r5bzbvda2ydnz09c6vxvqhxsmh37nhpw-dylint-driver-5.0.0/bin/dylint-driver \
  /nix/store/h3zdp7y2wknxxc0krs8x5gl7qhxzzkkv-octet-0.1.0/lib/liboctet.so \
  /nix/store/1yvh3d6y3fj3xk2dgwczrp1dj5svd92c-rust-default-1.96.0-nightly-2026-03-21 \
  /nix/store/alss3i1cgnh3g74gvgv3gvw1j0klx379-nickel-1.17.0/bin/nickel
```

That output now exists; use a fresh absolute output for another replay.
The helper asserts exit codes, relevant diagnostics, silence on clean cases, no ICE markers, and equal input identities.
Only unknown-lint, fragile-match, and FSM-exhaustiveness denial are enabled in these targeted probes.
This is not the full consumer gate.

## Complete-command diagnostic

Task 10435 confirmed that the old library fails the corrected doc-spoof expectation by incorrectly producing no diagnostics.
Molten task 10434 ran the unchanged canonical command on frozen `ab3efbe9788643caddd7f22bb0a13eca0d90426f`, changing only the library from run 14.
It reported 30 node-host errors and zero warnings, exit 2/Cargo 101; the summary is byte-identical to run 14.
Recorded identities matched, tracked source stayed unchanged, and no ICE markers appeared.
This is not complete workspace coverage. No production sealing markers or fallbacks were added; startup remains unauthorized and no normal-node VM launched.

## Retained tool

- Library: `/nix/store/h3zdp7y2wknxxc0krs8x5gl7qhxzzkkv-octet-0.1.0/lib/liboctet.so`
- BLAKE3: `61c02bb43a8c890d13f8f1d212b8ef7082e212bcae9ab86796163af55e09f228`
- Derivation: `/nix/store/508nv3pkijnfyhzvvmjd21q5d3b4wg5i-octet-0.1.0.drv`
- Build entry: `verification/exact-markers/check.nix`, not the default package output.
- Private evidence: `~/.local/state/onix/molten-node-vm/exact-markers/`.

A private GC root retains the library; local content verification passed.
This does not establish signatures, reproducibility, a production runtime cohort, or startup authority.
