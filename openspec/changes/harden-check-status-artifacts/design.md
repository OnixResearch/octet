## Context

`tigerstyle` already has the right high-level contract on paper: warning-only
runs should stay non-fatal, lint failures should be distinct from integration
failures, and structured output should stay valid even on error paths. The
actual runner does not yet uphold that contract.

`cargo-tigerstyle/src/engine.rs` still returns `1` whenever the child cargo
status is non-success or any finding exists, so a clean structured run can be
reported as failure even when SARIF has zero results and the summary says
`Findings: 0`. `tests/consumer_integration.rs` also still encodes the stale
compile-error expectation (`exit 1`) on JSON/SARIF error paths, so spec, tests,
and runtime behavior drift together.

Aspen review churn showed a second gap: the runner emits findings, but it does
not emit one canonical run-status artifact bundle. Reviewers had to infer what
happened from summary text, ad-hoc logs, and shell-level capture instead of a
self-describing status file plus structured-output sidecars.

## Goals / Non-Goals

**Goals:**
- restore a deterministic `0` / `1` / `2` outcome contract in the runner
- make the human summary, machine-readable status, and SARIF invocation metadata agree on the same run classification
- give downstream automation an optional artifact bundle that captures command, summary, status, and structured output without shell scraping
- update consumer-integration tests and docs so they enforce the same contract the runner ships

**Non-Goals:**
- adding new lints or changing existing lint severities
- replacing stdout/stderr as the default output surface when `--artifact-dir` is not used
- bundling downstream crate-test orchestration or OpenSpec evidence policy into the runner itself

## Decisions

### 1. Engine uses a typed run-outcome classifier

**Choice:** introduce a typed `RunOutcome` classifier in `cargo-tigerstyle/src/engine.rs` with explicit mappings for `clean`, `warning-only`, `lint-failure`, and `integration-failure`.

**Covers:** `Scenario: Clean structured run stays exit 0`; `Scenario: Cargo compilation failure uses integration-failure exit code`

**Rationale:** the current boolean collapse (`!status.success() || !findings.is_empty()`) throws away the distinction between child-process failure and lint severity. A typed classifier makes the exit-code contract inspectable and reusable by the summary, status sidecar, and SARIF emission paths.

**Alternative considered:** keep the current boolean exit logic and special-case clean structured runs after SARIF emission. Rejected because it preserves the same hidden coupling and still leaves compile-failure vs lint-failure ambiguous.

**Implementation:** while parsing diagnostics, accumulate warning/error/autofixable counts separately from child cargo completion. After `child.wait()`, map those counts plus child status into `RunOutcome`, then derive the final process exit code (`0`, `1`, or `2`) from that enum instead of from a raw boolean.

**Boundary cases:** warning-only findings still exit `0`; signaled or no-exit-code child termination stays `integration-failure` even when no findings were emitted; compile-failure paths still preserve valid structured stdout for the selected format.

### 2. `--artifact-dir` becomes the canonical sidecar surface

**Choice:** add `--artifact-dir <dir>` to `cargo-tigerstyle check` and persist deterministic sidecars: `status.json`, `command.txt`, `summary.txt`, and structured-output mirrors for `json` / `sarif` runs.

**Covers:** `Scenario: Artifact directory captures a structured run`; `Scenario: Artifact directory records integration failure cleanly`

**Rationale:** downstream reviewers and agents need a single run directory they can cite. A directory bundle is stronger than a lone status file because it keeps the command line, summary, and selected structured output together.

**Alternative considered:** add only `--status-json <path>`. Rejected because it still leaves the caller to invent filenames and capture rules for summary text and structured outputs.

**Implementation:** extend `cargo-tigerstyle/src/cli.rs` to parse `--artifact-dir` before `--`, create the directory if missing, and overwrite known files for the current run. `status.json` is written from the `RunOutcome` plus finding counts; `summary.txt` is emitted from the same summary renderer used for stderr; structured-output mirrors are written only for `json` and `sarif` modes.

**Boundary cases:** failed runs still write `status.json` and `summary.txt` once the engine reaches classification; `human` mode may omit a structured-output mirror; existing stdout/stderr behavior stays unchanged when `--artifact-dir` is absent.

### 3. Summary and SARIF must carry the same status contract

**Choice:** extend the stderr summary with explicit status and per-severity counts, and extend SARIF with `runs[0].invocations[0]` plus run properties that mirror the same classification exposed in `status.json`.

**Covers:** `Scenario: SARIF carries invocation metadata`; `Scenario: Clean structured run stays exit 0`

**Rationale:** the same run should not look clean in one place and ambiguous in another. Self-describing SARIF matters for code-scanning and saved evidence; self-describing summaries matter for humans reading stderr directly.

**Alternative considered:** keep status metadata only in `status.json`. Rejected because standalone SARIF files and shell-only stderr captures would still require an external sidecar to explain the run outcome.

**Implementation:** update `cargo-tigerstyle/src/output.rs` so summaries begin with a stable `Status:` line plus warning/error/autofixable counts, and teach the SARIF builder to emit invocation metadata and run-level properties that include status, exit code, and count fields.

**Boundary cases:** clean SARIF still emits a valid document with zero results and one invocation record; integration-failure SARIF stays valid JSON even when the underlying cargo check failed.

### 4. Consumer integration tests become the contract lock

**Choice:** update `tests/consumer_integration.rs` to assert the new exit-code contract, artifact-dir schema, and SARIF invocation metadata on both clean and compile-failure paths.

**Covers:** `Scenario: Clean structured run stays exit 0`; `Scenario: Cargo compilation failure uses integration-failure exit code`; `Scenario: Artifact directory captures a structured run`; `Scenario: Artifact directory records integration failure cleanly`; `Scenario: SARIF carries invocation metadata`

**Rationale:** this repo already has consumer-integration fixtures for warning-only and invalid-metadata paths. Extending those fixtures is the cheapest way to prevent another silent drift between spec and implementation.

**Alternative considered:** rely on unit tests around a new classifier helper. Rejected because CLI parsing, summary rendering, artifact writing, and SARIF output are all end-to-end behaviors.

**Implementation:** replace stale `exit 1` error-path assertions with `exit 2`, add artifact-dir fixture helpers that read saved sidecars, and add explicit assertions for summary status text plus SARIF invocation metadata.

**Boundary cases:** warning-only fixtures must keep exiting `0`; lockfile-restore and metadata-misuse cases keep the existing `exit 2` behavior and should not be regressed while the classifier is refactored.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Clean structured run stays exit 0`
- artifact path:
  - `openspec/changes/harden-check-status-artifacts/evidence/clean-sarif-status.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration clean_structured_run_stays_exit_zero -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test clean_structured_run_stays_exit_zero ... ok` and the saved assertions prove exit code `0`, summary status `clean`, and `results[].len() == 0`
- ordering / prerequisites:
  - land the `RunOutcome` classifier and clean-run fixture before saving the transcript
- cleanup / postcondition:
  - no temporary artifact dir remains outside the saved evidence path

- scenario or requirement:
  - `Scenario: Cargo compilation failure uses integration-failure exit code`
- artifact path:
  - `openspec/changes/harden-check-status-artifacts/evidence/error-path-exit-code.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration cargo_compilation_failure_uses_integration_failure_exit_code -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test cargo_compilation_failure_uses_integration_failure_exit_code ... ok` and the saved assertions prove exit code `2`, valid structured stdout, and summary status `integration-failure`
- ordering / prerequisites:
  - update the compile-failure fixture expectations after the classifier lands
- cleanup / postcondition:
  - no stale error-path fixture output remains in temporary dirs

- scenario or requirement:
  - `Scenario: Artifact directory captures a structured run`
- artifact path:
  - `openspec/changes/harden-check-status-artifacts/evidence/artifact-dir-structured-run.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_captures_structured_run -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_captures_structured_run ... ok` and the saved assertions prove the artifact dir contains exactly `status.json`, `command.txt`, `summary.txt`, and `results.sarif`, with matching status/count fields between `status.json` and `summary.txt`
- ordering / prerequisites:
  - implement `--artifact-dir` before saving the transcript
- cleanup / postcondition:
  - the test-owned temp directory is removed after the transcript is saved

- scenario or requirement:
  - `Scenario: Artifact directory records integration failure cleanly`
- artifact path:
  - `openspec/changes/harden-check-status-artifacts/evidence/artifact-dir-error-path.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration artifact_directory_records_integration_failure_cleanly -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test artifact_directory_records_integration_failure_cleanly ... ok` and the saved assertions prove `status.json` reports `integration-failure` while the structured-output mirror remains valid JSON for the selected format
- ordering / prerequisites:
  - reuse the compile-failure fixture after `--artifact-dir` support lands
- cleanup / postcondition:
  - only the saved transcript remains under `evidence/`

- scenario or requirement:
  - `Scenario: SARIF carries invocation metadata`
- artifact path:
  - `openspec/changes/harden-check-status-artifacts/evidence/sarif-invocation.txt`
- command / helper:
  - `nix develop -c bash -lc 'cargo test --test consumer_integration sarif_carries_invocation_metadata -- --nocapture'`
- exact expected output / acceptance signal:
  - the transcript contains `test sarif_carries_invocation_metadata ... ok` and the saved assertions prove `runs[0].invocations[0]` exists with the command line and exit code, and run properties expose the same status classification as `status.json`
- ordering / prerequisites:
  - land SARIF invocation metadata after the sidecar status schema is finalized so both use the same field names
- cleanup / postcondition:
  - no temporary SARIF fixture file remains outside the test tempdir

- scenario or requirement:
  - human/oracle checkpoint for the full change
- artifact path:
  - `openspec/changes/harden-check-status-artifacts/evidence/design-gate.txt`
- command / helper:
  - `openspec_gate stage=design change=harden-check-status-artifacts`
- exact expected output / acceptance signal:
  - the transcript reports a passing design-stage gate result with no blocker findings about exit-code ambiguity, artifact schema ambiguity, or missing scenario coverage
- ordering / prerequisites:
  - run after proposal, consumer-integration delta spec, design, and tasks are all finalized
- cleanup / postcondition:
  - the saved transcript references the active change path only

## Risks / Trade-offs

- more CLI surface area -> keep `--artifact-dir` optional and preserve existing stdout/stderr defaults when it is absent
- status metadata drift across summary, sidecar, and SARIF -> drive all three from one `RunOutcome` structure and one count aggregation path
- output-contract overlap with the active `structured-check-output` change -> land this change after that syncs or rebase the same files in lockstep so the two contracts do not fork
