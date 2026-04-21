# Consumer Integration Specification

## Purpose

Tiger Style consumer-integration requirements for the first-class
`cargo tigerstyle check` engine, metadata contract, lockfile safety, and
supported downstream entry points.
## Requirements
### Requirement: First-Class Consumer Check Command

The system MUST classify `cargo tigerstyle check` runs as
clean-or-warning-only, lint-failure, or integration-failure.

Clean runs and warning-only runs MUST exit `0`. Error-level lint findings MUST
exit `1`. Integration/configuration failures, including underlying
`cargo check` compilation failures, MUST exit `2`.

The trailing summary MUST report the same classification as any emitted
machine-readable run-status surface. When the run emits machine-readable
run-status data (`status.json` or SARIF run properties), that machine-readable
surface MUST report the same classification and exit code as the process
result.

The trailing summary block MUST use this exact line order:
- first line `--- tigerstyle summary ---`
- `Status: <status>`
- `Findings: <count>`
- `Warnings: <count>`
- `Errors: <count>`
- `Autofixable: <count>`
- when findings are non-zero, blank-line-separated `By crate:`, `By lint:`,
  and `Index:` sections in that order
- final line `---`

The summary `Status:` value MUST serialize exactly as `clean`, `warning-only`,
`lint-failure`, or `integration-failure`.

#### Scenario: Clean structured run stays exit 0

- GIVEN a workspace with zero tigerstyle findings
- WHEN `cargo tigerstyle check --output-format sarif` completes
- THEN the exit code is `0`
- AND the trailing summary reports status `clean`
- AND the emitted SARIF document has zero results

#### Scenario: Warning-only structured run stays exit 0

- GIVEN a workspace with warning-level tigerstyle findings and zero
  error-level tigerstyle findings
- WHEN `cargo tigerstyle check --output-format json` completes
- THEN the exit code is `0`
- AND the trailing summary reports status `warning-only`
- AND stdout remains valid newline-delimited JSON

#### Scenario: Error-level lint findings use exit 1

- GIVEN a workspace with at least one error-level tigerstyle finding and no
  cargo-child failure
- WHEN `cargo tigerstyle check --output-format json` completes
- THEN the exit code is `1`
- AND the trailing summary reports status `lint-failure`

#### Scenario: Cargo compilation failure uses integration-failure exit code

- GIVEN a workspace where the underlying `cargo check` fails to compile after
  any earlier lint diagnostics are emitted
- WHEN `cargo tigerstyle check --output-format json` or `--output-format sarif`
  completes
- THEN the exit code is `2`
- AND structured stdout remains valid for the selected format
- AND the trailing summary reports an `integration-failure` status

#### Scenario: Temp-workspace launch keeps the consumer cwd

- GIVEN a temp consumer workspace and a launcher that invokes `cargo-tigerstyle`
  through an explicit `--manifest-path` or a built binary path
- WHEN the launcher runs from a nested subdirectory inside that consumer
  workspace
- THEN the run resolves `[workspace.metadata.tigerstyle]` from the consumer
  workspace
- AND the supported launch path does not rely on `cargo run -p cargo-tigerstyle`
  against the consumer workspace

### Requirement: Workspace Metadata Owns Runner Configuration

Consumer-specific tigerstyle runner configuration MUST live in
`[workspace.metadata.tigerstyle]` in the consumer workspace `Cargo.toml`.

That metadata MUST define the no-argument check behavior through these keys:

- `default_scope`
- `cargo_check_args`

Tigerstyle MUST read that metadata when `cargo tigerstyle check` runs without
explicit overrides.

When the caller supplies explicit `cargo check` tail args after `--`, tigerstyle
MUST use that explicit tail and MUST NOT merge metadata `cargo_check_args` into
that invocation.

Invalid or incomplete `[workspace.metadata.tigerstyle]` runner metadata MUST
fail with a user-facing configuration error instead of silently falling back to
copied-script behavior.

Consumer-owned `dylint.toml` MUST remain the place for lint tuning such as
thresholds, disabled lints, and lint levels; runner configuration MUST NOT
require script-body edits.

#### Scenario: No-argument check uses workspace metadata

- **WHEN** a consumer runs `cargo tigerstyle check` with no scope flags
- **THEN** tigerstyle uses the default scope and cargo-check args from
  `[workspace.metadata.tigerstyle]`

#### Scenario: Explicit cargo-check args replace metadata defaults

- **WHEN** a consumer runs `cargo tigerstyle check -p crate-a -- --all-targets`
- **THEN** tigerstyle uses the explicit `--all-targets` tail for that run
- **AND** it does not merge metadata `cargo_check_args` into the same command

#### Scenario: Lint tuning stays in dylint.toml

- **WHEN** a consumer changes lint thresholds or disabled lints
- **THEN** they update `dylint.toml`
- **AND** they do not need to edit runner logic in a copied script

### Requirement: First-Class Nix Consumer Integration

Tigerstyle MUST provide a first-class Nix integration path for downstream Nix
consumers that uses the same core engine and workspace metadata contract as
`cargo tigerstyle check`.

That Nix integration MUST:

- expose a flake app for `nix run`
- expose a first-party flake helper for consumer `checks`
- avoid copied consumer shell scripts as the supported path
- allow a consumer flake to wire tigerstyle checks into both `nix run` and
  `nix flake check`
- call the same Rust engine used by `cargo tigerstyle check`

#### Scenario: Nix consumer runs tigerstyle through flake app

- **WHEN** a Nix-based consumer invokes the published tigerstyle flake app
- **THEN** the run uses the same configured scope and policy as
  `cargo tigerstyle check`

#### Scenario: Nix consumer wires tigerstyle into flake checks

- **WHEN** a Nix-based consumer adds tigerstyle to its flake checks
- **THEN** it uses the first-party Nix integration surface instead of a copied
  wrapper script

### Requirement: Repo-Local Entry Points Stay Thin

Tigerstyle MUST treat every published repo-local consumer entry point as a thin
delegation layer over the first-class tigerstyle engine.

Any repo-local tigerstyle entry point published for downstream consumers — such
as `just`, `cargo xtask`, CI helper commands, or generated local wrappers —
MUST be a thin delegation layer over the first-class tigerstyle engine.

Tigerstyle MUST provide a verification fixture, generated helper, or equivalent
reviewable artifact proving that documented helper entry points delegate without
reimplementing policy.

Those entry points MUST NOT reimplement rollout logic such as:

- toolchain detection
- driver bootstrap
- lockfile guarding
- default-scope policy
- metadata parsing

#### Scenario: just target delegates to the canonical command

- **WHEN** a consumer publishes `just tigerstyle`
- **THEN** that target delegates to `cargo tigerstyle check` or the equivalent
  first-party Nix entry point
- **AND** it does not carry a second copy of tigerstyle rollout logic

#### Scenario: CI helper stays policy-thin

- **WHEN** a consumer adds tigerstyle to CI
- **THEN** the CI command delegates to the same first-class engine used locally
- **AND** changes to tigerstyle rollout behavior happen in one implementation
  instead of many repo-local scripts

### Requirement: Run Outcome Artifacts

The system MUST optionally persist a reviewable run artifact bundle when the
caller passes `--artifact-dir <dir>` to `cargo tigerstyle check`.

The artifact bundle MUST include:
- `status.json` with these exact top-level keys: `status`, `exit_code`,
  `output_format`, `total_findings`, `warning_findings`, `error_findings`,
  `autofixable_findings`, and `cargo_process_exit`
- `status.json.exit_code` as an integer with allowed values `0`, `1`, or `2`,
  always equal to the final `cargo tigerstyle check` process exit code
- `status.json.status` serialized as exactly one of `clean`, `warning-only`,
  `lint-failure`, or `integration-failure`
- `status.json.output_format` serialized as exactly one of `human`, `json`, or
  `sarif`
- `status.json.cargo_process_exit` as an object with exact keys
  `classification` and `code`, where `classification` is exactly one of
  `success`, `exit-code`, or `terminated`, and `code` is an integer or `null`
- `status.json.cargo_process_exit.classification` as `success` when the cargo
  child exits `0`, `exit-code` when the cargo child exits with a non-zero
  integer code, and `terminated` when the cargo child has no exit code
- `status.json.total_findings`, `status.json.warning_findings`,
  `status.json.error_findings`, and `status.json.autofixable_findings` as
  non-negative integers
- `command.txt` with the exact top-level command line used for the run as one
  POSIX `sh`-compatible quoted line plus a trailing newline; arguments matching
  the safe character class `[A-Za-z0-9._/:=@-]+` stay unquoted, and every
  other argument is single-quoted, with each embedded `'` escaped by closing
  the single-quoted string, inserting a double-quoted single quote, and
  reopening the single-quoted string
- `summary.txt` containing a byte-for-byte copy of the trailing summary block
  written to stderr
- `results.jsonl` when `--output-format json` is selected, as the exact
  newline-delimited JSON-lines stdout stream for the run
- `results.sarif` when `--output-format sarif` is selected, as the exact SARIF
  stdout document for the run

A run artifact bundle is valid only when every required sidecar for its
selected `output_format` exists and no extra sidecar file is present:
`human` requires only `command.txt`, `status.json`, and `summary.txt`; `json`
requires only those three files plus `results.jsonl`; `sarif` requires only
those three files plus `results.sarif`. Missing any required sidecar or
adding any extra sidecar file makes the bundle invalid.

If `--artifact-dir` does not resolve to a usable writable artifact directory
before required sidecar writes begin, the run MUST exit `2`, and callers MUST
treat any would-be artifact bundle from that path as invalid. When
`--artifact-dir` is present, the runner MUST finalize the summary and any
machine-readable stdout document only after the artifact-write attempt has
settled so emitted metadata still matches the final process exit code. If a
required sidecar write fails after sidecar emission has already begun, the
final process exit code MUST still be `2`, the trailing summary and any emitted
machine-readable output MUST report that final integration-failure result, and
the runner MUST leave at least one required sidecar absent so the resulting
bundle is invalid under the presence-based validity rule.

When output format is `sarif`, the SARIF document MUST also embed:
- `runs[0].invocations[0].commandLine` and `runs[0].invocations[0].exitCode`,
  where `runs[0].invocations[0].exitCode` equals the final `cargo tigerstyle
  check` process exit code
- run properties with these exact keys: `status`, `exitCode`,
  `totalFindings`, `warningFindings`, `errorFindings`,
  `autofixableFindings`, `cargoProcessExitClassification`, and
  `cargoProcessExitCode`, where `exitCode` equals the final
  `cargo tigerstyle check` process exit code, `totalFindings`,
  `warningFindings`, `errorFindings`, and `autofixableFindings` are
  non-negative integers, and `cargoProcessExitCode` mirrors
  `status.json.cargo_process_exit.code`
- `runs[0].properties.status` serialized with the same allowed values as
  `status.json.status`

#### Scenario: Artifact directory captures a structured run

- GIVEN a caller provides `--artifact-dir <dir>` and `--output-format sarif`
- WHEN a run completes
- THEN `<dir>/status.json`, `<dir>/command.txt`, `<dir>/summary.txt`, and
  `<dir>/results.sarif` exist
- AND `status.json` and `summary.txt` agree on the exit classification and
  finding counts

#### Scenario: Artifact directory captures a JSON run

- GIVEN a caller provides `--artifact-dir <dir>` and `--output-format json`
- WHEN a warning-only run completes
- THEN `<dir>/status.json`, `<dir>/command.txt`, `<dir>/summary.txt`, and
  `<dir>/results.jsonl` exist
- AND `results.jsonl` is a valid newline-delimited JSON stream for the run
- AND `status.json.output_format` is `json`

#### Scenario: Command line quoting is deterministic

- GIVEN a top-level command line that contains safe arguments, arguments with
  spaces, embedded single quotes, and an empty argument
- WHEN `command.txt` is rendered
- THEN arguments matching `[A-Za-z0-9._/:=@-]+` remain unquoted
- AND arguments with spaces are single-quoted
- AND embedded single quotes are escaped by closing the single-quoted string,
  inserting a double-quoted single quote, and reopening the single-quoted
  string
- AND the empty argument renders as `''`

#### Scenario: Artifact directory records integration failure cleanly

- GIVEN a caller provides `--artifact-dir <dir>` and the underlying
  `cargo check` fails after any earlier diagnostics are emitted
- WHEN the run completes
- THEN `status.json` and `summary.txt` still exist
- AND the selected structured-output mirror remains valid for its format
- AND `status.json` reports an `integration-failure` status

#### Scenario: Artifact directory captures a human run

- GIVEN a caller provides `--artifact-dir <dir>` without `--output-format`
- WHEN a warning-only run completes
- THEN `<dir>/status.json`, `<dir>/command.txt`, and `<dir>/summary.txt`
  exist
- AND `<dir>/results.jsonl` and `<dir>/results.sarif` do not exist
- AND `status.json.output_format` is `human`

#### Scenario: Artifact directory setup failure exits 2

- GIVEN a caller provides `--artifact-dir <path>` and `<path>` cannot be used
  as a writable artifact directory before required sidecar writes begin
- WHEN the run reaches artifact emission
- THEN the process exits `2`
- AND the trailing summary reports `integration-failure`
- AND callers treat any would-be artifact bundle from that path as invalid

#### Scenario: Bundle shape mismatch is invalid

- GIVEN a would-be artifact bundle for `human`, `json`, or `sarif`
- WHEN any required sidecar for that `output_format` is missing or any extra
  sidecar file is present
- THEN callers treat the bundle as invalid

#### Scenario: Sidecar write failure after emission exits 2

- GIVEN a caller provides `--artifact-dir <dir>` and a required sidecar write
  fails after some required sidecars have already been written
- WHEN the run completes
- THEN the final process exit code is `2`
- AND the trailing summary reports `integration-failure`
- AND any emitted machine-readable output still reports the final process exit
  code
- AND the runner leaves at least one required sidecar absent
- AND callers treat the artifact bundle as invalid

#### Scenario: Terminated cargo child records terminated classification

- GIVEN a caller provides `--artifact-dir <dir>` and the cargo child
  terminates without an exit code
- WHEN the run completes
- THEN the final process exit code is `2`
- AND `status.json.cargo_process_exit.classification` is `terminated`
- AND `status.json.cargo_process_exit.code` is `null`

#### Scenario: SARIF carries invocation metadata

- GIVEN `cargo tigerstyle check --output-format sarif` completes
- WHEN a consumer inspects the SARIF run object
- THEN `runs[0].invocations[0]` records the command line and exit code
- AND the SARIF run properties expose the same status classification as
  `status.json`

