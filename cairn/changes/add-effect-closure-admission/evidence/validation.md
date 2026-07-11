# Effect-closure admission validation

Validation date: 2026-07-11

## Implemented boundary

`cargo octet admit --profile deterministic-core --closure -- --target TARGET` now runs the existing strict-profile parent first, then computes conservative selected source-token closure. V1:

- recognizes `alloc`, `panic`, `unsafe`, `clock`, `random`, `fs`, `net`, `spawn`, `lock`, `ffi`, and `unknown`;
- permits only `alloc` under `deterministic-core`;
- requires exact function-addressed summaries for every non-macro call not resolved to one selected local function;
- grants no name-based intrinsic exemption for `Ok`, `Err`, `Some`, `Vec`, or similar spellings;
- marks every macro invocation `unknown` because arbitrary macro tokens are not expanded;
- requires an explicit Cargo target rather than trusting host or ambient Cargo configuration;
- propagates selected-local effects to a finite fixed point, retaining cycles;
- binds receipt identity to the direct parent, profile/config, source/function addresses, policy, Cargo graph/lock, providers, target/toolchain, and normalized command;
- verifies canonical receipt structure and the complete strict-profile parent before checking closure bundle integrity.

BLAKE3 integrity remains identity/linkage evidence, not signer authenticity, authorization, or resistance to coherent replacement of every trust root.

## Focused positive and negative checks

| Command / check | Outcome |
| --- | --- |
| `nix develop -c cargo test -p cargo-octet --lib effect_closure` | PASS; 12 tests covering catalog/policy validation, local propagation, cycles, exact provider summaries, stale/unknown edges, no intrinsic exemption, roles, addresses, trait dispatch, macros, cfg, FFI, unsafe, panic, deterministic identity, and coherent/non-coherent tampering. |
| `nix develop -c cargo test -p cargo-octet --lib admit::tests` | PASS; three tests covering feature/explicit-target selection, workspace path confinement, complete bundle verification, strict-parent verification, and summary tampering. |
| `nix develop -c cargo test -p cargo-octet --lib cli::tests::admission` | PASS; run/verify parsing plus required profile/closure/artifact and output-format negatives. |
| `nix develop -c cargo test --test consumer_integration effect_closure_admission_stops_after_failed_direct_check` | PASS; failed direct admission does not invoke metadata or emit closure evidence. |
| `nix develop -c cargo test -p cargo-octet --lib` | PASS; 314 tests. |
| `nix develop -c cargo test --test consumer_integration` | PASS; 78 tests, including packaged standalone runner/cache smokes. |
| `nix develop -c cargo fmt --all --check` | PASS. |
| `git diff --check` | PASS. |
| `nix build .#checks.x86_64-linux.effect-closure-catalog --no-link -L` | PASS; Nickel contract positive/negative fixtures and generated JSON freshness. |
| `nix run .#octet-policy-validate` | PASS. |
| `nix build .#cargo-octet --no-link -L` | PASS. Final reviewed implementation package used for dogfood: `/nix/store/2hfqxksl89f8a872vgci4mricwdzhh04-cargo-octet-0.1.0`. Local builds emitted a failed optional SSH-builder connection to `10.10.10.1` and then completed locally. |

## Valence dogfood

Valence retains exact evidence under `../valence/evidence/octet/effect-closure/`.

The initial no-summary run:

- passed the direct deterministic-core parent;
- selected two production source files and 16 function nodes;
- denied with exit 1, 78 `call_edge_unknown` findings, and 90 total diagnostics;
- used zero reviewed providers;
- retained receipt `b3:5a8f90f5963368ca25b4d3b44a68fbe4b338d3d4a420ece302dc55bc41da5e7f`.

The final run:

- used 78 exact sorted summaries and five provider identities (`core`, `alloc`, `std`, `blake3`, `serde_json`);
- bound explicit target `x86_64-unknown-linux-gnu`;
- admitted 16 function nodes with only `alloc`, zero closure diagnostics, profile identity `b3:217467f897307ee25135762e38bde9d321e5c3cca5ac7b4322adea7b37f95539`, config identity `b3:47531b6882b7e11f0091faf7321a2ebf42219f93567a21bec33fa1deb5d1e7c3`, and receipt `b3:7787045860bfd88d8c82bf8c30f6253f989d673cc7c11003fe11b4e838705ff7`;
- independently verified valid with no diagnostics;
- repeated with byte-identical `effect-closure.json` and `status.json`;
- rejected copied closure-summary and strict-profile-parent-summary tampering with exit 1 and deterministic hash/linkage diagnostics.

Valence formatting, workspace tests, all-target Clippy, Nickel policy freshness, the dedicated Nix policy check, and `nix flake check path:$PWD -L` all passed on `x86_64-linux`; Nix reported `aarch64-linux` omitted as incompatible.

## Known repository-wide baselines

These failures predate and are outside this change; they remain visible rather than being suppressed:

- `nix develop -c cargo test --lib`: FAIL with the same eight lint inventory/snapshot/catalog tests (117 passed, eight failed).
- `nix develop -c cargo clippy -p cargo-octet --lib -- -D warnings`: FAIL only on 15 existing dead-code findings in `cargo-octet/src/imported_proof.rs`; the changed `admit.rs` and `effect_closure.rs` surfaces are clean after their Clippy findings were corrected.
- `nix develop -c cargo clippy --workspace --all-targets -- -D warnings`: FAIL on existing dead code plus unchanged broad-workspace findings such as metadata `must_use` candidates and the source-acquisition nested condition.
- `nix flake check -L`: FAIL in the existing Clippy rail; the captured tail includes unchanged `octet-lint-metadata` `must_use_candidate` findings.
- `nix run path:.#cairn -- tracey coverage --root . --json`: FAIL repository-wide with 1,069 requirements, zero referenced IDs, zero dangling IDs, and `valid: false`. The repository Tracey configuration still does not recognize source references, so this is not presented as change-local coverage success.

## Lifecycle checks

- `nix run path:.#cairn -- validate --root .`: PASS with 68 accepted specs, eight active changes, and no issues.
- Proposal, design, and tasks gates for `add-effect-closure-admission`: PASS with no issues.
- The pinned CLI does not support newer `strict` or `implementation` stage names; no unsupported gate is claimed.

## Claim boundary

The implementation and dogfood establish deterministic, fail-closed selected source-token closure under exact reviewed assertions and recorded inputs. They do not establish complete name resolution, macro expansion, trait/dynamic dispatch, MIR call-graph semantics, dependency implementation behavior, runtime reachability or ordering, panic freedom, semantic determinism, compiler correctness, verifier soundness, whole-program correctness, authenticity, authorization, or release eligibility.
