## Phase 1: Implementation

- [ ] I1 Refactor `cargo-tigerstyle/src/engine.rs` to compute a typed `RunOutcome` and enforce the `0` / `1` / `2` exit-code contract for clean-or-warning-only, lint-failure, and integration-failure runs.
- [ ] I2 Update `cargo-tigerstyle/src/output.rs` so the stderr summary reports explicit status plus warning/error/autofixable counts, and the SARIF emitter carries invocation and run-status metadata.
- [ ] I3 Extend `cargo-tigerstyle/src/cli.rs` and runner plumbing with `--artifact-dir`, then write `status.json`, `command.txt`, `summary.txt`, and structured-output mirrors for `json` / `sarif` runs.
- [ ] I4 Update `tests/consumer_integration.rs`, `README.md`, and `docs/consumer-guide.md` so fixtures, expectations, and consumer docs match the hardened run-status and artifact-bundle contract.

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Clean structured run stays exit 0" with `cargo test --test consumer_integration clean_structured_run_stays_exit_zero -- --nocapture`; save output at `openspec/changes/harden-check-status-artifacts/evidence/clean-sarif-status.txt`
- [ ] V2 Verify scenario "Cargo compilation failure uses integration-failure exit code" with `cargo test --test consumer_integration cargo_compilation_failure_uses_integration_failure_exit_code -- --nocapture`; save output at `openspec/changes/harden-check-status-artifacts/evidence/error-path-exit-code.txt`
- [ ] V3 Verify scenario "Artifact directory captures a structured run" with `cargo test --test consumer_integration artifact_directory_captures_structured_run -- --nocapture`; save output at `openspec/changes/harden-check-status-artifacts/evidence/artifact-dir-structured-run.txt`
- [ ] V4 Verify scenario "Artifact directory records integration failure cleanly" with `cargo test --test consumer_integration artifact_directory_records_integration_failure_cleanly -- --nocapture`; save output at `openspec/changes/harden-check-status-artifacts/evidence/artifact-dir-error-path.txt`
- [ ] V5 Verify scenario "SARIF carries invocation metadata" with `cargo test --test consumer_integration sarif_carries_invocation_metadata -- --nocapture`; save output at `openspec/changes/harden-check-status-artifacts/evidence/sarif-invocation.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`openspec_gate stage=design change=harden-check-status-artifacts`; input=`final proposal.md plus consumer-integration delta spec and design.md`; asserts=`design-stage review accepts the exit-code, artifact-dir, and SARIF-invocation contract without ambiguity`; evidence=`openspec/changes/harden-check-status-artifacts/evidence/design-gate.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, I4, V1, V2, V3, V4, V5, H1
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: Clean structured run stays exit 0` -> `V1` -> `openspec/changes/harden-check-status-artifacts/evidence/clean-sarif-status.txt`
- `Scenario: Cargo compilation failure uses integration-failure exit code` -> `V2` -> `openspec/changes/harden-check-status-artifacts/evidence/error-path-exit-code.txt`
- `Scenario: Artifact directory captures a structured run` -> `V3` -> `openspec/changes/harden-check-status-artifacts/evidence/artifact-dir-structured-run.txt`
- `Scenario: Artifact directory records integration failure cleanly` -> `V4` -> `openspec/changes/harden-check-status-artifacts/evidence/artifact-dir-error-path.txt`
- `Scenario: SARIF carries invocation metadata` -> `V5` -> `openspec/changes/harden-check-status-artifacts/evidence/sarif-invocation.txt`
