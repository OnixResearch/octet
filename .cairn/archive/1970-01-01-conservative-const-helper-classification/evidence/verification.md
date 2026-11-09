# Classifier precision evidence

## Scope and original observation

The reported eight-finding deny-all failure is `/tmp/hwr-component-proof-20261016.5bwWjw/bounded-profile-octet.json`. It was read, not rerun. Two `missing_const_fn` findings at component.rs:66 and :79 violate the canonical non-const-call guardrail. The other six are not silently removed.

## Native admission before implementation

Executed from the Octet root with `/nix/store/ynjawpza9ci9i8ha7f3ywaqwbd60m83k-cairn-0.1.0/bin/cairn`:

| Command | Verdict | Receipt hash | Input hash |
| --- | --- | --- | --- |
| `cairn gate proposal conservative-const-helper-classification --root .` | PASS | `4d97459fa0bc41364a5f4288045aac53bcfbbcfcbf07b579227b8115c7240059` | `941eaa676f8ff3565961565eda382008515531737ac7990f006d735dc906ed31` |
| `cairn gate design conservative-const-helper-classification --root .` | PASS | `bc2510ed42dda305acc40fbc2585a5bd77690ca026994548b559dfa015d92634` | `a196a5d550d98dd7013d3887acd07f1b88aa51f6c14282a3cb23633d1b1b1737` |
| `cairn gate tasks conservative-const-helper-classification --root .` | PASS | `0b24b0ad350bd7cbcf4709bab62f9349e9defba417b940caabebf40ff9a5a701` | `c82b57c3230dfa2d6f4b373922aff8d888fb67e3b881f7ffeab39f1fb9e99f8e` |

Policy `octet-default`; policy hash `62e40f642e798bfaf5a68a495a7456bcf4e9fd8106ae2f52a24100967b39df5d`. All returned `valid: true`, no issues, advisory evidence mode and empty review receipt hashes. These are native structural admission receipts, not human-review, implementation, release, publication, or runtime proof. Design wording was subsequently clarified to enumerate the exact three type names and one function name; final gate receipts are saved separately.

## Exercised scoped smoke

The retained shell reported `IN_NIX_SHELL=impure`, pinned nightly `2026-03-21`, store rustc `/nix/store/1yvh3d6y3fj3xk2dgwczrp1dj5svd92c-rust-default-1.96.0-nightly-2026-03-21/bin/rustc`.

A temporary rustc-driver probe compiled the exact source-extracted `const_subset_type` and `ConstSubsetVisitor`, applying the same free-function and primitive-signature guards, against the complete checked-in `ui_tests/missing_const_fn.rs`. The fixture compiled successfully. Observed results:

```text
SUBSET scale_value true
SUBSET main false
SUBSET identifier false
SUBSET identifier::{closure#0} false
SUBSET versioned_interface false
SUBSET runtime_value true
SUBSET comparison_with_call false
SUBSET overloaded_add false
SUBSET <Value as Scale>::scale false
SUBSET reads_static false
```

`runtime_value` is inside the HIR subset but has no source arithmetic hint, so this probe does not claim it emits a diagnostic. The probe intentionally tests the changed HIR classifier, not complete Octet registration or UI snapshots. An initial probe incorrectly queried closure signatures before the free-function guard and hit a rustc ICE; applying the actual implementation's guard before the query fixed the probe. The owning implementation already had that guard ordering. Temporary probe files are not permanent tests.

The pinned compiler also compiled the complete existing fixture after changing
one named helper at a time from `fn` to `const fn`: `scale_value` compiled (exit
0); `identifier`, `versioned_interface`, `comparison_with_call`, and
`overloaded_add` each failed as expected (exit 1). Full diagnostics and the exact
transformation/command are in `const-semantic-compile.txt`. This validates the
paired fixture semantics only; it is not owning-lint registration or UI proof.

## Historical environment blockers

- `nix develop -c bash -c 'CARGO_TARGET_DIR=/tmp/octet-const-classifier-20261016 cargo build -p octet --lib --locked'`: failed before compilation because unrelated `nix/public-dependencies.nix` is untracked and omitted by Git flake source selection.
- `nix develop path:. -c bash -c 'CARGO_TARGET_DIR=/tmp/octet-const-classifier-20261016 cargo build -p octet --lib --locked'`: failed before compilation because `vendor/dependencies.json` is absent.
- In the already-retained pinned devshell, `env CARGO_TARGET_DIR=/tmp/octet-const-classifier-20261016 cargo build -p octet --lib --locked`: workspace dependency resolution attempted unavailable bounded-exec revision `44eb74eb639194665ee820562d5b2ce2c3ff6a94`. Ambient Git credential helpers unexpectedly attempted authentication and failed; this path was not repeated or used as authorization. Source acquisition and credentials remain parent-owned.
- An offline direct full-library rustc attempt using preexisting shared target dependencies failed due incompatible compiler/dependency artifacts. No dependency override, source pin change, suppression, or compatible-build claim was made.

## Completed actual owning proof after intake

After PublicInputs completed the existing exact-source intake/check, Main
authorized actual full-source verification in the already-retained exact owning
Nix environment. The retained normal vendor config has bounded-exec's SSH source
key, whereas the current manifest/lock uses HTTPS. Main explicitly authorized a
transient additional Cargo source-replacement table mapping that exact HTTPS
revision to the same existing verified directory source and checksums. No package,
version, revision or implementation override was introduced. Every executed
command disabled Git credential helpers, askpass and SSH, and Cargo was offline.

- Actual full owning library build: exit 0, fresh target
  `/tmp/octet-const-owning-proof-20261016`; receipt `owning-library-build.txt`.
- Actual existing owning UI driver source build: exit 0, separate fresh target
  `/tmp/octet-const-owning-driver-20261016`; receipt `owning-driver-build.txt`.
- Actual complete `missing_const_fn` fixture using that new library and driver:
  exit 0, exactly the unchanged checked-in snapshot, one primitive-arithmetic
  advisory and no findings for the negative cases; receipt `owning-ui-fixture.txt`.

`owning-proof-commands.txt` preserves the exact exercised commands, source-mapping
content, authorization boundary and environment. The retained helper
`verify-owning-const-fixture.py` applies the existing UI harness's driver flags,
normalization and exact snapshot comparison to this one existing fixture. Artifact
identities are in `owning-ui-fixture.txt`; source identities are in
`owning-proof-inputs.txt`. Temporary fixture directories and source mapping were
removed after execution. No pending Nix proof jobs remain.

Fresh Nix entry did not settle and was canceled before choosing this route;
an earlier binding query timed out with a busy SQLite eval-cache database. This
is actual full-source/registration/UI proof in the retained owning environment,
not a fresh-environment or anonymous/public acquisition claim. The full ignored
UI suite, workspace/project gates and downstream HWR deny-all replay remain
parent-owned and were not run here. Archive and publication remain parent-owned.

## Remaining consumer obligations

- F1/F2/F3/F7: `ComponentLimits`, `ComponentProfile`, `AdmittedComponentProfile`, `admit_component_profile` repeat `component` as the explicit ancestor-word rule specifies. Do not rename public API merely for style; a real compatibility contract may be documented through the existing rule, otherwise API/policy ownership must decide.
- F6: `versioned_interface` is a nontrivial 27-line parser with zero assertions versus required two. Fallible rejection is not currently an assertion-density exemption. Meaningful internal invariants/refactoring or a separately authorized policy decision are required; redundant input assertions are not a precision fix.
- F8: `blake3::Hasher::new()` returns an external ADT. Octet self-architecture's reviewed `pure-value` provider fact exists, but is not an authorized HWR fact or an input to the lint. The exact blocker is a reviewed project-specific provider-to-construction classification contract, not lack of a BLAKE3 spelling exception. No broad exemption or new abstraction was invented.

Octet's third-party standards-prose publication hold is unchanged. No remote mutation or anonymous-checkout proof is claimed.
