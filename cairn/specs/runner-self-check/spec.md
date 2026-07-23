# Runner Self Check Specification

## Purpose

Defines the `runner-self-check` capability.

## Requirements

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

#### Scenario: Non-zero Cargo child is propagated
r[octet.runner_self_check.failure.child_failure]
- GIVEN valid canonical metadata and a configured Cargo sentinel that exits non-zero
- WHEN the runner invokes that child
- THEN it SHALL report an integration-failure summary and SHALL exit with integration code `2` rather than returning success.

### Requirement: Self-check claims SHALL remain bounded
r[octet.runner_self_check.boundary] Octet SHALL describe this gate as pinned local static-lint execution evidence over the configured workspace library scope.

#### Scenario: Passing check does not become a correctness claim
r[octet.runner_self_check.boundary.non_claims]
- GIVEN the root metadata, positive self-check, and malformed negative all validate
- WHEN contributors review the result
- THEN Octet SHALL NOT claim test, binary, example, bench, dependency, macro-expansion, runtime, whole-program correctness, lint completeness, or release-eligibility coverage beyond the exact configured and observed run.

### Requirement: Canonical structured findings SHALL use portable locators
r[octet.runner_self_check.inventory] The packaged self-check SHALL emit machine-readable findings whose artifact locators are deterministic and repository-relative.

#### Scenario: Workspace paths normalize before rendering
r[octet.runner_self_check.inventory.paths]
- GIVEN Cargo reports an absolute diagnostic path inside the discovered workspace root
- WHEN the runner constructs and validates the finding
- THEN the locator SHALL be normalized to the equivalent repository-relative path before structured rendering or baseline capture.
- AND exact workspace-root substrings in diagnostic text SHALL be replaced by a portable relative root before stable-ID construction.
- AND absolute locators outside the workspace, empty locators, and parent traversal SHALL fail closed rather than leak or broaden path authority.

### Requirement: Octet SHALL ratchet its self-check warning budget
r[octet.runner_self_check.warning_budget] Octet SHALL keep a reviewed machine-readable baseline for the canonical workspace-library self-check and SHALL fail when a current finding has a stable ID absent from that baseline.

#### Scenario: Reviewed warning debt remains non-blocking
r[octet.runner_self_check.warning_budget.baseline]
- GIVEN the checked-in canonical baseline was generated from a successful complete Cargo child run
- WHEN the unchanged self-check runs with that baseline
- THEN it SHALL exit `0`, report zero new IDs and zero stale IDs, and retain warning-only status without claiming the accepted findings are correct.

#### Scenario: New warning fails the ratchet
r[octet.runner_self_check.warning_budget.regression]
- GIVEN a writable source copy and the checked-in canonical baseline
- WHEN one additional warning finding is introduced without changing metadata or lint policy
- THEN the check SHALL exit `1` and report exactly one new stable ID rather than returning success or integration code `2`.

#### Scenario: One bounded category is reduced before capture
r[octet.runner_self_check.warning_budget.cleanup]
- GIVEN the initial canonical inventory contains one `negated_predicate` finding
- WHEN the underlying local name is rewritten into positive form without suppression or behavior change
- THEN the final canonical inventory SHALL contain zero `negated_predicate` findings and 2,670 total accepted warning occurrences.

#### Scenario: Baseline evidence remains bounded
r[octet.runner_self_check.warning_budget.boundary]
- GIVEN the baseline and positive/negative gate pass
- WHEN contributors review self-check status
- THEN the result SHALL establish only stable-ID membership for the pinned configured static-lint run.
- AND it SHALL NOT claim warning correctness, test or binary coverage, dependency behavior, runtime safety, whole-program correctness, lint completeness, or release eligibility.
