# Cargo Octet Self Check Delta

## ADDED Requirements

### Requirement: Octet SHALL declare canonical root runner metadata
r[octet.runner_self_check.metadata] The Octet repository root SHALL provide the documented `[workspace.metadata.octet]` runner policy and SHALL NOT rely on implicit defaults, top-level legacy metadata, or command-line scope substitution.

#### Scenario: No-argument scope is broad and bounded
r[octet.runner_self_check.metadata.scoped]
- GIVEN the checked-in root Cargo workspace manifest
- WHEN the runner loads its canonical Octet metadata
- THEN `default_scope` SHALL be exactly `--workspace` and `cargo_check_args` SHALL be exactly `--lib`.
- AND the default SHALL cover every workspace library without claiming coverage of tests, binaries, examples, or benches.

### Requirement: The packaged canonical self-check SHALL execute successfully
r[octet.runner_self_check.execution] Octet SHALL retain deterministic evidence that its packaged no-argument check proceeds beyond metadata admission and completes the configured Cargo child process without policy-weakening flags.

#### Scenario: Canonical Nix app completes the self-check
r[octet.runner_self_check.execution.pass]
- GIVEN the pinned packaged runner, driver, lint library, toolchain, and checked-in root metadata
- WHEN `nix run .#cargo-octet -- check` runs without scope overrides or `--allow-style-violations`
- THEN it SHALL exit `0` after a successful Cargo child process and SHALL emit a valid Octet human summary with no error-level findings.

### Requirement: Invalid metadata SHALL continue to fail before Cargo
r[octet.runner_self_check.failure] Restoring repository self-check configuration SHALL NOT weaken the runner's fail-closed consumer metadata boundary.

#### Scenario: Malformed canonical array is refused pre-Cargo
r[octet.runner_self_check.failure.malformed]
- GIVEN a workspace whose `[workspace.metadata.octet]` `default_scope` contains a non-string value
- WHEN the packaged runner attempts a no-argument check
- THEN it SHALL exit with integration code `2`, emit the stable malformed-array diagnostic, and SHALL NOT invoke the configured Cargo sentinel.

### Requirement: Self-check claims SHALL remain bounded
r[octet.runner_self_check.boundary] Octet SHALL describe this gate as pinned local static-lint execution evidence over the configured workspace library scope.

#### Scenario: Passing check does not become a correctness claim
r[octet.runner_self_check.boundary.non_claims]
- GIVEN the root metadata, positive self-check, and malformed negative all validate
- WHEN contributors review the result
- THEN Octet SHALL NOT claim test, binary, example, bench, dependency, macro-expansion, runtime, whole-program correctness, lint completeness, or release-eligibility coverage beyond the exact configured and observed run.
