# Verification

Date: 2026-10-22

## Implemented boundary

- `docs/proof-implementation-gap.md` names the four specification assumptions verbatim: Synchronous process assumption, Request-to-response correlation assumption, Atomic persistence of related protocol state, and Formal coverage of the entire protocol, including extensions.
- The same page states that a verified model is not a verified implementation, and names the three layers: interleaving, fault handling, and unverified extensions.
- Each layer names the system-level testing that closes it: schedule-controlled interleaving testing of the running implementation, fault injection at the persistence and I/O boundaries, and extension conformance and differential testing.
- The page states the honest claim boundary: it does not claim that any existing proof is unsound, and it states that this repository does not currently run the system-level testing that closes the three layers.
- `docs/law-tested-cores.md` states that law and property tests over a pure core do not close the three layers and links the class.
- `README.md`, `docs/trust-boundary-evidence.md`, `docs/contributor-guide.md`, and `docs/standards/source/verification.adoc` link the class.
- The accepted capability `proof-implementation-gap` records the class, and the accepted `law-tested-deterministic-cores` requirement states that law tests are never presented as implementation evidence.

## Evidence

- `cairn gate proposal document-proof-implementation-gap --root .` returned verdict `PASS` with zero issues.
- `cairn gate design document-proof-implementation-gap --root .` returned verdict `PASS` with zero issues.
- `cairn gate tasks document-proof-implementation-gap --root .` returned verdict `PASS` with zero issues.
- `cairn validate --root .` returned `valid: true` with zero issues before the change and zero issues after the sync.
- `cairn validate --root . --strict` returned `valid: true` with zero issues before the change.
- `cairn sync document-proof-implementation-gap --root .` reported `blocked: false` with two `sync_delta_spec` actions, and the executed sync reported `mutated: true` and wrote `.cairn/specs/proof-implementation-gap/spec.md` plus the modified `.cairn/specs/law-tested-deterministic-cores/spec.md`.
- A content check over `docs/proof-implementation-gap.md` found all four assumption names, all three layer names, the model-versus-implementation sentence, and both claim-boundary statements.
- A relative-link check over `docs/proof-implementation-gap.md`, `docs/law-tested-cores.md`, `docs/trust-boundary-evidence.md`, and `docs/contributor-guide.md` resolved every relative Markdown link target.
- A scan replicating the overclaim checkers in `tests/docs_positioning.rs` over `README.md`, `docs/trust-boundary-evidence.md`, `docs/law-tested-cores.md`, `docs/contributor-guide.md`, `docs/proof-implementation-gap.md`, and `docs/standards/source/verification.adoc` reported zero findings.
- `nix build .#checks.x86_64-linux.docs-examples --no-link -L` completed with exit status 0.
- `cargo test --offline --test docs_positioning` passed 23 tests with 0 failures. The run used a byte-identical copy of the changed documentation, because the workspace cannot be built in place in this environment.
- `cargo test --offline -p octet-standards` passed 24 tests with 0 failures in the last test binary, including the advisory CLI report tests that carry the standards source anchors.

## Environment notes

- `nix develop -c bash -c 'cargo --version'` fails in this checkout. The vendored cairn package cannot be installed: `sed: couldn't open temporary file /nix/store/kz354q3my9hbhyibdrabmabk2358mz5v-cairn-0.1.0/share/licenses/cairn/LICENSES/sedwiIe6Q: Permission denied`, followed by `error: Cannot build '/nix/store/2fz0sf49wlg5ki8f5z0jp8glm5xd0w7y-cairn-0.1.0.drv'. Reason: builder failed with exit code 1.` The gates above therefore ran with a cairn binary rebuilt from an unmodified `vendor/sources/cairn` source tree in a scratch copy of this repository, where one packaging line in `nix/public-dependencies.nix` was given a write permission for the copied license files. No file in this repository was changed for that rebuild.
- The `cairn` binary on `PATH` is a different build from the vendored one. With it, `cairn gate tasks` fails for every change in this checkout with `failed to parse policy cairn-policy/generated/cairn-policy.json: cairn.workflow.profile.registry at workflow_profile_policy.schemas: initial workflow profile is missing: outcome-machine`. The same failure reproduces on the pre-existing active change `detect-false-durability-claims`, so it is not caused by this change.
- `nix/`, `nix/public-dependencies.nix`, and `vendor/` are untracked in this checkout. They were made visible to the Nix flake source with `git add -N`, the step documented in `AGENTS.md`, and the intent-to-add index entries were removed afterwards.
- The two Cargo test runs used a scratch copy of this repository whose changed documentation files are byte-identical to the files here, because the workspace cannot be built in place here: the git dependency `bounded-exec` at revision `44eb74eb639194665ee820562d5b2ce2c3ff6a94` is not in the local Cargo git cache and cannot be fetched without credentials.

## Non-claims

This change documents a failure class. It does not claim that any proof is unsound, it does not extend any proof to an implementation, and it does not claim that the system-level interleaving, fault-injection, or extension-conformance testing named for the three layers exists in this repository.

The deterministic checks recorded above are documentation and lifecycle checks. They do not prove implementation correctness, whole-program behavior, or release eligibility.
