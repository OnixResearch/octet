## ADDED Requirements

### Requirement: Run Outcome Artifacts

The system MUST optionally persist a reviewable run artifact bundle when the
caller passes `--artifact-dir <dir>` to `cargo tigerstyle check`.

The artifact bundle MUST include:
- `status.json` with `status`, `exit_code`, `output_format`, `total_findings`,
  `warning_findings`, `error_findings`, `autofixable_findings`, and the
  underlying cargo-process exit classification
- `command.txt` with the exact top-level command line used for the run
- `summary.txt` containing the same trailing summary block written to stderr
- a mirror of the selected structured output (`results.jsonl` or
  `results.sarif`) when `--output-format json` or `sarif` is selected

When output format is `sarif`, the SARIF document MUST also embed invocation
metadata that records the command line, exit code, and run-status
classification so the SARIF file is self-describing without the sidecar
summary.

#### Scenario: Artifact directory captures a structured run

- GIVEN a caller provides `--artifact-dir <dir>` and `--output-format sarif`
- WHEN a run completes
- THEN `<dir>/status.json`, `<dir>/command.txt`, `<dir>/summary.txt`, and
  `<dir>/results.sarif` exist
- AND `status.json` and `summary.txt` agree on the exit classification and
  finding counts

#### Scenario: Artifact directory records integration failure cleanly

- GIVEN a caller provides `--artifact-dir <dir>` and the underlying
  `cargo check` fails after any earlier diagnostics are emitted
- WHEN the run completes
- THEN `status.json` and `summary.txt` still exist
- AND the selected structured-output mirror remains valid for its format
- AND `status.json` reports an `integration-failure` status

#### Scenario: SARIF carries invocation metadata

- GIVEN `cargo tigerstyle check --output-format sarif` completes
- WHEN a consumer inspects the SARIF run object
- THEN `runs[0].invocations[0]` records the command line and exit code
- AND the SARIF run properties expose the same status classification as
  `status.json`

## MODIFIED Requirements

### Requirement: First-Class Consumer Check Command

The system MUST classify `cargo tigerstyle check` runs as
clean-or-warning-only, lint-failure, or integration-failure.

Clean runs and warning-only runs MUST exit `0`. Error-level lint findings MUST
exit `1`. Integration/configuration failures, including underlying
`cargo check` compilation failures, MUST exit `2`.

The trailing summary and any machine-readable run-status output MUST report the
same classification and exit code.

#### Scenario: Clean structured run stays exit 0

- GIVEN a workspace with zero tigerstyle findings
- WHEN `cargo tigerstyle check --output-format sarif` completes
- THEN the exit code is `0`
- AND the trailing summary reports status `clean`
- AND the emitted SARIF document has zero results

#### Scenario: Cargo compilation failure uses integration-failure exit code

- GIVEN a workspace where the underlying `cargo check` fails to compile after
  any earlier lint diagnostics are emitted
- WHEN `cargo tigerstyle check --output-format json` or `--output-format sarif`
  completes
- THEN the exit code is `2`
- AND structured stdout remains valid for the selected format
- AND the trailing summary reports an `integration-failure` status
