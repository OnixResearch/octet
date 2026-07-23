# Runner Self Check Delta

## MODIFIED Requirements

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

#### Scenario: Build-shell owner paths reduce one bounded cohort
r[octet.runner_self_check.warning_budget.reduction]
r[octet.runner_self_check.warning_budget.reduction.build_shell]
- GIVEN the reviewed canonical inventory contains 1,114 `non_trait_imports` occurrences represented by 372 stable identities, including four build-script identities
- WHEN the build shell replaces `std::env`, `std::fs`, `std::path::Path`, and `std::path::PathBuf` imports with explicit owner-qualified use sites
- THEN the canonical inventory SHALL contain 2,666 total warning occurrences, 1,816 unique stable identities, and 1,110 `non_trait_imports` occurrences.
- AND `std::fmt::Write as _` SHALL remain a trait-resolution import.
- AND the change SHALL NOT add an alias, lint suppression, exception, policy change, or unrelated behavior change.

#### Scenario: Old baseline proves monotonic removal before refresh
r[octet.runner_self_check.warning_budget.reduction.old_baseline]
- GIVEN the qualified build shell and the previously accepted 1,820-identity baseline
- WHEN the canonical self-check evaluates current findings before baseline replacement
- THEN it SHALL exit `0` with warning-only status, exactly four stale IDs, and zero new IDs.

#### Scenario: Reviewed refresh contains removals only
r[octet.runner_self_check.warning_budget.reduction.refresh]
- GIVEN the old-baseline reduction result is accepted for review
- WHEN the runner writes the replacement baseline
- THEN its stable-ID set SHALL equal the old set minus exactly the four reviewed build-shell identities and SHALL contain no additions.
- AND the unchanged self-check SHALL report zero new IDs and zero stale IDs against the replacement.

#### Scenario: Permanent regression semantics remain unchanged
r[octet.runner_self_check.warning_budget.reduction.regression]
- GIVEN the refreshed lower baseline and the hermetic one-warning source mutation
- WHEN the permanent positive and negative self-check cases run
- THEN accepted warning-only source SHALL exit `0` and the one-new-ID mutation SHALL exit `1` with exactly one new stable identity.
- AND malformed metadata and Cargo-child failures SHALL retain integration exit `2`.

#### Scenario: Baseline evidence remains bounded
r[octet.runner_self_check.warning_budget.boundary]
r[octet.runner_self_check.warning_budget.reduction.boundary]
- GIVEN the monotonic baseline refresh and positive/negative gate pass
- WHEN contributors review self-check status
- THEN the result SHALL establish only removal and membership of stable IDs for the pinned configured static-lint run.
- AND it SHALL NOT claim warning correctness, build-script correctness, test or binary coverage, dependency behavior, runtime safety, whole-program correctness, lint completeness, or release eligibility.
