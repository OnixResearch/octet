# Runner Self Check Delta

## ADDED Requirements

### Requirement: Canonical structured findings SHALL use portable locators
r[octet.runner_self_check.inventory] The packaged self-check SHALL emit machine-readable findings whose artifact locators are deterministic and repository-relative.

#### Scenario: Workspace paths normalize before rendering
r[octet.runner_self_check.inventory.paths]
- GIVEN Cargo reports an absolute diagnostic path inside the discovered workspace root
- WHEN the runner constructs and validates the finding
- THEN the locator SHALL be normalized to the equivalent repository-relative path before structured rendering or baseline capture.
- AND absolute paths outside the workspace, empty locators, and parent traversal SHALL fail closed rather than leak or broaden path authority.

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
- THEN the final baseline SHALL contain zero `negated_predicate` findings and 2,670 total accepted warnings.

#### Scenario: Baseline evidence remains bounded
r[octet.runner_self_check.warning_budget.boundary]
- GIVEN the baseline and positive/negative gate pass
- WHEN contributors review self-check status
- THEN the result SHALL establish only stable-ID membership for the pinned configured static-lint run.
- AND it SHALL NOT claim warning correctness, test or binary coverage, dependency behavior, runtime safety, whole-program correctness, lint completeness, or release eligibility.
