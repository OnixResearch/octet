# Verification

Date: 2026-09-01

## Implemented boundary

- The typed benchmark profile has BLAKE3 identity `b3:64e4b8163b3c89d8143d4f6ff07230d33b2b2a9b7bcab35bf45325027a20cd47`.
- The profile binds four separate modes, five samples per mode, fixed Cargo metadata arguments, limits, environment facts, and non-claims.
- The profile binds one synthetic corpus and immutable Bounded Exec revision `44eb74eb639194665ee820562d5b2ce2c3ff6a94`.
- The bounded shell clears the child environment and uses process-group teardown.
- Cold mode resets its dedicated cache. Warm mode performs an unmeasured preparation run.
- No-change and incremental modes record exact source-state identities without source mutation.
- The shell requires explicit kernel, CPU, governor, load, toolchain, Nix closure, host, and isolation facts.
- The selected shell records wall duration and output bytes. It records CPU duration and peak memory as unsupported.
- The pure core preserves failed samples and reports improved, stable, regressed, inconclusive, or incomparable results.
- Allocation evidence remains scoped to explicit instrumented intervals.
- LLVM copy-site evidence preserves constant lengths and unresolved lengths separately.
- The signal oracle binds case, source, lint, location, outcome, reviewer role, and rationale.
- Public claims bind the projection, corpus, environment, sample range, and interpretation boundary.
- All added Rust source files contain fewer than 300 lines.

## Evidence

- `nix develop --builders '' -c cargo test -p cargo-octet --lib benchmark::` passed 9 tests. One fixture-regeneration test remained ignored.
- The focused matrix covers stable, improved, regressed, noisy, failed, cold, warm, no-change, incremental, allocation, copy-site, and signal cases.
- Negative cases cover arbitrary commands, sample limits, hidden allocation, instrumentation drift, compiler drift, unsupported observations, duplicate oracles, unknown copy lengths, and public overclaims.
- `nix develop --builders '' -c cargo clippy -p cargo-octet --lib --tests -- -D warnings` passed.
- The 43 focused evidence-rail tests passed.
- Six evidence-contract tests passed. One regeneration test remained ignored after the generated contract was refreshed.
- The focused artifact-bundle tampering test passed.
- The benchmark profile, policy registry, and Cairn traceability Nix checks passed with local builders.
- The checked import produced a manual-review receipt with stable performance and explicit signal findings.
- `cargo octet artifact verify` reported the benchmark bundle as valid and remeasured all seven benchmark artifacts.
- The negative CLI probe rejected free-form `--command` authority with integration exit 2 before execution.
- The broad cargo-octet library suite passed 438 tests. It retained two inherited Nickel evaluator failures and six ignored tests.

## Inherited blockers

The canonical `cargo-octet-self-check` remains an integration failure on both this change and untouched `origin/main`.

Both runs reported status 2, 89 findings, 89 warnings, zero errors, 1,813 accepted baseline identities, zero current accepted identities, 81 new identities, and 1,813 stale identities.

The focused wrapper also converted an inherited compiler internal error into an invalid severity diagnostic. A separate target and `CARGO_INCREMENTAL=0` did not change the result.

The broad library failures remain:

- `config::tests::active_architecture_pointer_loads_fresh_policy`
- `config::tests::architecture_pointer_rejects_escape_and_stale_manifest`

Both reject the available Nickel evaluator because it is not `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.

This change does not replace a baseline, suppress a finding, weaken a gate, or claim a successful Octet self-check.

## Non-claims

Benchmark evidence covers only selected workloads, source states, modes, environments, samples, resource scopes, and signal corpora.

It does not prove correctness, universal performance, corpus representativeness, formal proof, whole-program behavior, or release eligibility.
