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
r[octet.runner_self_check.warning_budget] Octet SHALL keep a reviewed machine-readable baseline for the canonical workspace-library self-check, SHALL fail when a current finding has a stable ID absent from that baseline, and SHALL admit debt reductions only through reviewed monotonic refreshes.

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

#### Scenario: Initial positive-naming cleanup remains preserved
r[octet.runner_self_check.warning_budget.cleanup]
- GIVEN the initial canonical inventory contained one `negated_predicate` finding
- WHEN the underlying local name was rewritten into positive form without suppression or behavior change
- THEN subsequent canonical inventories SHALL retain zero `negated_predicate` findings.
- AND the historical first reviewed baseline SHALL remain documented as 2,670 warning occurrences before later monotonic reductions.

#### Scenario: CLI exit owner paths reduce one isolated identity
r[octet.runner_self_check.warning_budget.reduction]
r[octet.runner_self_check.warning_budget.reduction.cli_exit]
- GIVEN the reviewed canonical inventory contains 1,114 `non_trait_imports` occurrences represented by 372 stable identities, including one unique private `ExitCode` import in `cargo-octet/src/lib.rs`
- WHEN that import is replaced in place and the thin CLI shell uses explicit `std::process::ExitCode` owner paths
- THEN the canonical inventory SHALL contain 2,669 total warning occurrences, 1,819 unique stable identities, and 1,113 `non_trait_imports` occurrences.
- AND success, delegated, and integration-error exit values SHALL remain unchanged.
- AND the change SHALL NOT add an alias, lint suppression, exception, policy change, or unrelated behavior change.

#### Scenario: Old baseline proves first monotonic removal before refresh
r[octet.runner_self_check.warning_budget.reduction.old_baseline]
- GIVEN the owner-qualified CLI exit shell and the previously accepted 1,820-identity baseline
- WHEN the canonical self-check evaluates current findings before baseline replacement
- THEN it SHALL exit `0` with warning-only status, exactly one stale ID, and zero new IDs.
- AND the stale ID SHALL be the reviewed `non_trait_imports` identity `b3:8515683389cc483f`.

#### Scenario: First reviewed refresh contains removals only
r[octet.runner_self_check.warning_budget.reduction.refresh]
- GIVEN the old-baseline CLI exit reduction result is accepted for review
- WHEN the runner writes the replacement baseline
- THEN its stable-ID set SHALL equal the old set minus exactly `b3:8515683389cc483f` and SHALL contain no additions.
- AND the unchanged self-check SHALL report zero new IDs and zero stale IDs against the replacement.

#### Scenario: First permanent regression semantics remain unchanged
r[octet.runner_self_check.warning_budget.reduction.regression]
- GIVEN the refreshed lower CLI-exit baseline and the hermetic one-warning source mutation
- WHEN the permanent positive and negative self-check cases run
- THEN accepted warning-only source SHALL exit `0` and the one-new-ID mutation SHALL exit `1` with exactly one new stable identity.
- AND malformed metadata and Cargo-child failures SHALL retain integration exit `2`.

#### Scenario: Public unsafe ABI owner path reduces a second isolated identity
r[octet.runner_self_check.warning_budget.reduction.public_unsafe_abi]
- GIVEN the reviewed canonical inventory contains 1,113 `non_trait_imports` occurrences represented by 371 stable identities, including one unique private `ExternAbi` import in `src/safety/public_unsafe_api.rs`
- WHEN the import is removed and Rust ABI matching uses the explicit `rustc_abi::ExternAbi::Rust` owner path
- THEN the canonical inventory SHALL contain 2,668 total warning occurrences, 1,818 unique stable identities, and 1,112 `non_trait_imports` occurrences.
- AND positive Rust-ABI detection and negative extern-ABI exclusion behavior SHALL remain unchanged.
- AND the change SHALL NOT add an alias, lint suppression, exception, policy change, or unrelated behavior change.

#### Scenario: Old baseline proves the ABI-path removal before refresh
r[octet.runner_self_check.warning_budget.reduction.public_unsafe_abi.old_baseline]
- GIVEN the owner-qualified ABI classifier and the previously accepted 1,819-identity baseline
- WHEN the canonical self-check evaluates current findings before baseline replacement
- THEN it SHALL exit `0` with warning-only status, exactly one stale ID, and zero new IDs.
- AND the stale ID SHALL be the reviewed `non_trait_imports` identity `b3:672b1993827f233b`.

#### Scenario: Second reviewed refresh contains removals only
r[octet.runner_self_check.warning_budget.reduction.public_unsafe_abi.refresh]
- GIVEN the old-baseline ABI-path reduction result is accepted for review
- WHEN the runner writes the replacement baseline
- THEN its stable-ID set SHALL equal the old set minus exactly `b3:672b1993827f233b` and SHALL contain no additions.
- AND the unchanged self-check SHALL report zero new IDs and zero stale IDs against the replacement.

#### Scenario: ABI-path regression semantics remain unchanged
r[octet.runner_self_check.warning_budget.reduction.public_unsafe_abi.regression]
- GIVEN the refreshed lower ABI-path baseline and the hermetic one-warning source mutation
- WHEN focused ABI classification plus permanent positive and negative self-check cases run
- THEN Rust ABI detection and extern-ABI exclusion SHALL pass, accepted warning-only source SHALL exit `0`, and the one-new-ID mutation SHALL exit `1` with exactly one new stable identity.
- AND malformed metadata and Cargo-child failures SHALL retain integration exit `2`.

#### Scenario: Baseline evidence remains bounded
r[octet.runner_self_check.warning_budget.boundary]
r[octet.runner_self_check.warning_budget.reduction.boundary]
r[octet.runner_self_check.warning_budget.reduction.public_unsafe_abi.boundary]
- GIVEN the monotonic baseline refresh and positive/negative gate pass
- WHEN contributors review self-check status
- THEN the result SHALL establish only removal and membership of stable IDs for the pinned configured static-lint run.
- AND it SHALL NOT claim warning correctness, ABI-model completeness, lint correctness, test or binary coverage, dependency behavior, runtime safety, whole-program correctness, lint completeness, or release eligibility.
