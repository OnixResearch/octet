# Consumer Integration Specification

## Purpose

Defines the structured output contract for `cargo tigerstyle check`, enabling machine-readable diagnostics for LLM-driven triage, CI integration (SARIF), and human progress tracking (summary).
## Requirements
### Requirement: JSON Diagnostic Output
The system MUST support a `--output-format json` flag that emits one JSON object per diagnostic line to stdout, while rustc's human-readable output continues on stderr.

Each JSON object MUST contain the following fields with the specified types:
- `index`: string — finding identifier (`F1`, `F2`, ...) unique within the run
- `lint`: string — bare lint name (e.g., `no_unwrap`)
- `group`: string — domain group, one of `tigerstyle::safety`, `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`, `tigerstyle::concurrency`, or `tigerstyle::unknown` for unrecognized lints. This is an output-side label resolved from a static lookup table (`DIAGNOSTIC_METADATA` in `src/diagnostics.rs`), not dependent on Dylint/rustc group registration
- `severity`: string — effective severity, one of `warning`, `error`
- `file`: string — relative file path from workspace root
- `line`: integer — 1-based line number
- `column`: integer — 1-based column number
- `message`: string — primary diagnostic message text
- `suggestion`: string or null — suggested replacement text if the lint emits one
- `applicability`: string or null — one of `MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, or null
- `principle`: string — Tiger Style principle name sourced from `DIAGNOSTIC_METADATA` in `src/diagnostics.rs` (e.g., `Assertion Quality`, `Function Length`). If a lint fires that is not registered in `DIAGNOSTIC_METADATA`, the value MUST be `"unknown"`
- `crate_name`: string — name of the crate being checked, extracted from the `target.name` field in cargo's `compiler-message` JSON wrapper
- `version`: integer — schema version, initially `1`; incremented on breaking field changes

#### Scenario: LLM parses JSON findings into granular tasks
- GIVEN a workspace with findings from multiple lints across multiple crates
- WHEN `cargo tigerstyle check --output-format json` completes
- THEN stdout contains one valid JSON object per line, each parseable with `serde_json::from_str`, and every object has all required fields

#### Scenario: JSON output coexists with human stderr
- GIVEN `--output-format json` is specified
- WHEN the check runs
- THEN JSON findings appear on stdout (one per line), and stderr contains only the trailing summary block (no human-readable diagnostics in `json` mode, since stdout carries the structured data)

### Requirement: SARIF Output
The system MUST support a `--output-format sarif` flag that emits a SARIF v2.1.0 document to stdout.

The SARIF document MUST include:
- `$schema` set to the SARIF v2.1.0 JSON schema URI
- `version` set to `"2.1.0"`
- `runs` array with exactly one run object containing:
  - `tool.driver.name` set to `"cargo-tigerstyle"`
  - `tool.driver.version` set to the cargo-tigerstyle version
  - `tool.driver.rules[]` with one entry per distinct lint that fired, each with `id` (bare lint name), `shortDescription`, and a `properties.group` field
  - `results[]` with one entry per finding, each containing:
    - `ruleId` set to the bare lint name
    - `level` set to `"warning"` or `"error"`
    - `message.text` matching the primary diagnostic message
    - `locations[0].physicalLocation` with `artifactLocation.uri` (relative file path), `region.startLine`, and `region.startColumn`
    - `properties.findingIndex` set to the finding index string (e.g., `"F3"`)

#### Scenario: GitHub Code Scanning ingestion
- GIVEN a workspace with findings
- WHEN `cargo tigerstyle check --output-format sarif` completes
- THEN the stdout output is valid JSON conforming to SARIF v2.1.0 structure with `$schema`, `version`, `runs[0].tool.driver.name`, `runs[0].tool.driver.rules[]`, and `runs[0].results[]` fields present. Validation is structural spot-checks in Rust integration tests, not full JSON Schema validation

### Requirement: Run Summary
The system MUST print a closing summary to stderr at the end of every check run, regardless of output format.

The summary MUST include:
- `--- tigerstyle summary ---` opening delimiter and `---` closing delimiter for machine extraction
- total finding count
- per-crate finding count (sorted descending by count)
- per-lint finding count (sorted descending by count)
- count of autofixable findings (those with `MachineApplicable` suggestions)
- per-finding index table listing each finding's index, lint, crate, and file:line

#### Scenario: Summary after noisy run
- GIVEN a workspace with 15 findings across 3 crates from 4 different lints
- WHEN `cargo tigerstyle check` completes
- THEN stderr ends with a summary block showing crate counts, lint counts, total, and autofixable count

#### Scenario: Summary for clean run
- GIVEN a workspace with zero findings
- WHEN `cargo tigerstyle check` completes
- THEN stderr ends with a summary showing `Findings: 0` (no autofixable parenthetical for zero findings)

### Requirement: Error Path Behavior
The system MUST produce valid output on stdout even when the underlying `cargo check` encounters compilation errors (exit code 2 path).

- In `json` mode: any findings emitted before the error MUST appear as valid JSON lines; no partial/malformed JSON on stdout
- In `sarif` mode: the SARIF document MUST still be a valid JSON document with whatever findings were emitted before the error
- In all modes: the summary MUST still print on stderr, reflecting only the findings that were successfully emitted
- The exit code MUST be `2` when the underlying cargo check fails, regardless of output format

#### Scenario: Compilation error during JSON output
- GIVEN a workspace with a syntax error in one crate and lint findings in another
- WHEN `cargo tigerstyle check --output-format json` runs
- THEN stdout contains valid JSON lines for findings emitted before the error, stderr contains the compilation error, and exit code is 2

#### Scenario: Compilation error during SARIF output
- GIVEN a workspace with a compilation error
- WHEN `cargo tigerstyle check --output-format sarif` runs
- THEN stdout is a valid SARIF JSON document (possibly with zero results), and exit code is 2

### Requirement: Finding Index
The system MUST assign each finding a deterministic index (`F1`, `F2`, ...) based on order of appearance in the diagnostic stream.

The index MUST be:
- unique within a single run
- monotonically increasing
- assigned by order of arrival from the cargo subprocess diagnostic stream (not sorted)
- present in all output formats (JSON field, SARIF property, and prefixed in human summary)

Indices are run-scoped only and NOT stable across invocations. In multi-crate workspaces, cargo may emit diagnostics in different orders due to parallel compilation, so the same finding may receive different indices between runs.

#### Scenario: LLM references specific finding by index
- GIVEN a run that emits 10 findings
- WHEN an LLM reads the JSON output
- THEN it can reference `F3` and the index maps to exactly one finding with a specific file, line, and lint

### Requirement: Output Format Flag
The system MUST accept `--output-format {human,json,sarif}` on the `check` subcommand.

The default MUST be `human`, which preserves the current pass-through behavior identically.

#### Scenario: Default behavior unchanged
- GIVEN no `--output-format` flag is specified
- WHEN `cargo tigerstyle check` runs
- THEN the lint diagnostic messages on stderr are identical to the current behavior (rustc human-readable diagnostics with colors preserved on TTY), exit code is unchanged, and a new trailing summary block is appended to stderr after the diagnostic stream. Note: cargo progress/build-status output may differ slightly since the engine uses `--message-format=json` internally

#### Scenario: Invalid format rejected
- GIVEN `--output-format xml` is specified
- WHEN `cargo tigerstyle check` runs
- THEN the tool exits with code 2 and an error message listing valid formats

#### Scenario: Conflicting --message-format rejected
- GIVEN `cargo tigerstyle check -- --message-format=short` is specified
- WHEN the tool runs
- THEN the tool exits with code 2 and an error message explaining that `--message-format` is managed by `--output-format`

### Requirement: Fingerprint Command Surface [r[fingerprint-command-surface]]

The system MUST provide a `cargo tigerstyle fingerprint` command surface for validating and maintaining selected function fingerprints.

The command MUST support `--check`, `--write`, `--update <path-or-filter>`, and `--diff` modes. Check mode MUST fail on mismatches and invalid metadata. Write and update modes MUST modify only fingerprint attributes or marker comments, not function bodies. Diff mode MUST list reviewed functions whose computed hashes differ from stored hashes.

#### Scenario: Check reports matching fingerprints [r[fingerprint-command-surface.scenario.check-matching]]

- GIVEN a workspace with fingerprinted functions whose stored hashes match computed hashes
- WHEN `cargo tigerstyle fingerprint --check` runs
- THEN the command exits successfully
- AND it reports the number of checked functions

#### Scenario: Check fails on mismatch [r[fingerprint-command-surface.scenario.check-mismatch]]

- GIVEN a fingerprinted function whose body tokens changed after review
- WHEN `cargo tigerstyle fingerprint --check` runs
- THEN the command exits non-zero
- AND the diagnostic includes the function name, expected hash, actual hash, and update command hint

#### Scenario: Write mode updates markers only [r[fingerprint-command-surface.scenario.write-markers-only]]

- GIVEN a Rust file with a missing or stale function fingerprint marker
- WHEN `cargo tigerstyle fingerprint --write` runs
- THEN the command writes or updates the fingerprint marker
- AND it does not change the function body tokens

#### Scenario: Diff mode lists reviewed changes [r[fingerprint-command-surface.scenario.diff-reviewed-functions]]

- GIVEN multiple fingerprinted functions and one changed reviewed function
- WHEN `cargo tigerstyle fingerprint --diff` runs
- THEN the output lists the changed function, file path, old hash, new hash, and review class
- AND unchanged functions are not listed as changed

### Requirement: Fingerprint Marker Forms [r[fingerprint-marker-forms]]

The system MUST support both first-class attribute markers and stable comment markers for function fingerprints.

Attribute form MUST support `#[fingerprint("b3:...", review = "...")]` or an equivalent Tigerstyle-owned proc-macro path. Comment form MUST support an adjacent marker such as `// tigerstyle-fingerprint: b3:... review=...` immediately before the function. The checker MUST ignore its own marker when recomputing the function hash.

#### Scenario: Attribute marker validates [r[fingerprint-marker-forms.scenario.attribute-validates]]

- GIVEN a function annotated with a supported fingerprint attribute
- WHEN fingerprint validation runs
- THEN the checker parses the hash and review metadata from the attribute
- AND it validates the computed function hash against the stored hash

#### Scenario: Comment marker validates without proc macro [r[fingerprint-marker-forms.scenario.comment-validates]]

- GIVEN a stable Rust crate that uses an adjacent `// tigerstyle-fingerprint:` marker instead of a proc macro attribute
- WHEN fingerprint validation runs
- THEN the checker parses and validates the marker
- AND the crate does not need to add a fingerprint proc-macro dependency solely for validation

### Requirement: Fingerprint Machine Output [r[fingerprint-machine-output]]

The system MUST provide deterministic machine-readable output for fingerprint validation results when requested.

Machine output MUST include the schema version, function name, relative file path, review class, stored hash, computed hash, status, macro allowance state, and optional evidence or spec reference. It MUST NOT include absolute paths, timestamps, local usernames, or environment dumps.

#### Scenario: Machine output captures mismatch identity [r[fingerprint-machine-output.scenario.mismatch-identity]]

- GIVEN `cargo tigerstyle fingerprint --check --output-format json` finds a mismatch
- WHEN the command emits machine output
- THEN the JSON record includes function name, relative path, review class, stored hash, computed hash, and status `mismatch`
- AND it omits absolute paths and timestamps

#### Scenario: Inventory preserves evidence references [r[fingerprint-machine-output.scenario.inventory-evidence]]

- GIVEN fingerprint metadata includes a spec ID or evidence path
- WHEN machine inventory output is requested
- THEN the JSON record includes that reference unchanged
- AND the record remains deterministic across repeated runs with the same source tree

### Requirement: Content Addressed Finding Identity [r[content-addressed-finding-identity]]

The system MUST support optional content-addressed identity fields for Tigerstyle findings, including finding fingerprint, source hash, snippet hash, config hash, and tool revision.

#### Scenario: Finding hash fields appear on request [r[content-addressed-finding-identity-scenario-finding-hash-fields-appear-on-request]]

- GIVEN JSON output is requested with content-addressed fields enabled
- WHEN a lint finding is emitted
- THEN the JSON record includes stable hash fields without absolute paths or timestamps

#### Scenario: Line moves do not define identity [r[content-addressed-finding-identity-scenario-line-moves-do-not-define-identity]]

- GIVEN a finding moves line number while its normalized snippet and lint context remain stable
- WHEN fingerprints are compared
- THEN the finding identity remains attributable without relying on the old line number

### Requirement: Content Addressed Evidence Headers [r[content-addressed-evidence-headers]]

The system MUST define evidence transcript headers that pin command, config, tool revision, fixture hashes, and repository revision without recording secrets or unstable local environment values.

#### Scenario: Evidence header is deterministic [r[content-addressed-evidence-headers-scenario-evidence-header-is-deterministic]]

- GIVEN the same command, config, fixtures, tool revision, and repo revision
- WHEN an evidence header is regenerated
- THEN the header hashes are identical across repeated runs

### Requirement: Semantic Refactor Evidence Records [r[semantic-refactor-evidence-records]]

The system MUST define evidence records for rename and move changes that preserve symbol identity with old path, new path, symbol name, optional fingerprint, and review rationale.

#### Scenario: Move record preserves identity [r[semantic-refactor-evidence-records-scenario-move-record-preserves-identity]]

- GIVEN a reviewed function moves files without token changes
- WHEN semantic refactor evidence is generated
- THEN the record links old path, new path, symbol name, and fingerprint so reviewers can see identity preservation

#### Scenario: Rename record states rationale [r[semantic-refactor-evidence-records-scenario-rename-record-states-rationale]]

- GIVEN a symbol is renamed
- WHEN refactor evidence is captured
- THEN the record includes old name, new name, path context, and rationale rather than relying on diff noise alone

### Requirement: Refactor Evidence Machine Output [r[refactor-evidence-machine-output]]

The system MUST provide deterministic machine-readable refactor evidence that omits absolute paths, timestamps, usernames, and environment dumps.

#### Scenario: Machine output is stable [r[refactor-evidence-machine-output-scenario-machine-output-is-stable]]

- GIVEN the same refactor mapping and source tree
- WHEN refactor evidence is emitted twice
- THEN the output is identical except for intentionally changed records

### Requirement: Slotcar Canonical CLI Surface

The system MUST expose `cargo slotcar` / `cargo-slotcar` as the canonical downstream command surface for check runs, structured output, summaries, SARIF tool metadata, Nix package/app outputs, and consumer templates. If the old `cargo tigerstyle` / `cargo-tigerstyle` path remains available during transition, it MUST be documented as a compatibility alias and MUST delegate to the same implementation and output schema as `slotcar`.

The deterministic proof artifact `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt` MUST show the canonical binary/package surfaces include `slotcar` and that any supported compatibility alias exits successfully with equivalent behavior for a minimal fixture.

#### Scenario: Cargo subcommand uses slotcar

- GIVEN a consumer workspace with the repo-provided tool installed from this checkout
- WHEN the consumer runs `cargo slotcar check`
- THEN the command executes the same lint pipeline as the prior first-class cargo wrapper and reports the tool identity as `slotcar`

#### Scenario: Structured outputs name slotcar

- GIVEN a check run emits human summary, JSON, or SARIF output
- WHEN a consumer inspects tool-name fields, delimiters, or headings in the output
- THEN product/tool identity fields use `slotcar` instead of `tigerstyle`, except for explicitly documented compatibility or lint-namespace fields

#### Scenario: Compatibility alias is explicit

- GIVEN the implementation keeps `cargo tigerstyle` available during the rename transition
- WHEN a consumer invokes that old command path
- THEN it is documented and tested as a compatibility alias to the canonical `cargo slotcar` behavior rather than a second independent tool surface
