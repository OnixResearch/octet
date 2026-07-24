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

#### Scenario: Standards resource owner path reduces a third isolated identity
r[octet.runner_self_check.warning_budget.reduction.standards_resource]
- GIVEN the reviewed canonical inventory contains 1,112 `non_trait_imports` occurrences represented by 370 stable identities, including one unique private `check_resources` import in `crates/octet-standards/src/runner.rs`
- WHEN the import is removed and the pure runner calls the explicit `crate::resource::check_resources` owner path at the same orchestration point
- THEN the canonical inventory SHALL contain 2,667 total warning occurrences, 1,817 unique stable identities, and 1,111 `non_trait_imports` occurrences.
- AND bounded-resource acceptance, unbounded-resource rejection, checker ordering, and diagnostic behavior SHALL remain unchanged.
- AND the change SHALL NOT add an alias, lint suppression, exception, policy change, or unrelated behavior change.

#### Scenario: Old baseline proves the standards-resource removal before refresh
r[octet.runner_self_check.warning_budget.reduction.standards_resource.old_baseline]
- GIVEN the owner-qualified standards runner and the previously accepted 1,818-identity baseline
- WHEN the canonical self-check evaluates current findings before baseline replacement
- THEN it SHALL exit `0` with warning-only status, exactly one stale ID, and zero new IDs.
- AND the stale ID SHALL be the reviewed `non_trait_imports` identity `b3:3965b4dd27194e41`.

#### Scenario: Third reviewed refresh contains removals only
r[octet.runner_self_check.warning_budget.reduction.standards_resource.refresh]
- GIVEN the old-baseline standards-resource reduction result is accepted for review
- WHEN the runner writes the replacement baseline
- THEN its stable-ID set SHALL equal the old set minus exactly `b3:3965b4dd27194e41` and SHALL contain no additions.
- AND the unchanged self-check SHALL report zero new IDs and zero stale IDs against the replacement.

#### Scenario: Standards-resource regression semantics remain unchanged
r[octet.runner_self_check.warning_budget.reduction.standards_resource.regression]
- GIVEN the refreshed lower standards-resource baseline and the hermetic one-warning source mutation
- WHEN focused resource behavior, complete standards tests, and permanent positive and negative self-check cases run
- THEN bounded-resource acceptance and unbounded-resource rejection SHALL pass, accepted warning-only source SHALL exit `0`, and the one-new-ID mutation SHALL exit `1` with exactly one new stable identity.
- AND malformed metadata and Cargo-child failures SHALL retain integration exit `2`.

#### Scenario: Standards report owner path reduces a fourth isolated identity
r[octet.runner_self_check.warning_budget.reduction.standards_report]
- GIVEN the reviewed canonical inventory contains 1,111 `non_trait_imports` occurrences represented by 369 stable identities, including one unique private `CheckReport` import in `crates/octet-standards/src/runner.rs`
- WHEN the import is removed and the pure runner uses explicit `crate::report::CheckReport` owner paths in its return type and report construction
- THEN the canonical inventory SHALL contain 2,666 total warning occurrences, 1,816 unique stable identities, and 1,110 `non_trait_imports` occurrences.
- AND report fields, field order, diagnostic order, positive serialization, and negative blocking-diagnostic behavior SHALL remain unchanged.
- AND the change SHALL NOT add an alias, lint suppression, exception, policy change, or unrelated behavior change.

#### Scenario: Old baseline proves the standards-report removal before refresh
r[octet.runner_self_check.warning_budget.reduction.standards_report.old_baseline]
- GIVEN the owner-qualified standards runner and the previously accepted 1,817-identity baseline
- WHEN the canonical self-check evaluates current findings before baseline replacement
- THEN it SHALL exit `0` with warning-only status, exactly one stale ID, and zero new IDs.
- AND the stale ID SHALL be the reviewed `non_trait_imports` identity `b3:b918c07b0ac7abe7`.

#### Scenario: Fourth reviewed refresh contains removals only
r[octet.runner_self_check.warning_budget.reduction.standards_report.refresh]
- GIVEN the old-baseline standards-report reduction result is accepted for review
- WHEN the runner writes the replacement baseline
- THEN its stable-ID set SHALL equal the old set minus exactly `b3:b918c07b0ac7abe7` and SHALL contain no additions.
- AND the unchanged self-check SHALL report zero new IDs and zero stale IDs against the replacement.

#### Scenario: Standards-report regression semantics remain unchanged
r[octet.runner_self_check.warning_budget.reduction.standards_report.regression]
- GIVEN the refreshed lower standards-report baseline and the hermetic one-warning source mutation
- WHEN focused report behavior, complete standards tests, and permanent positive and negative self-check cases run
- THEN positive serialization and negative blocking-diagnostic behavior SHALL pass, accepted warning-only source SHALL exit `0`, and the one-new-ID mutation SHALL exit `1` with exactly one new stable identity.
- AND malformed metadata and Cargo-child failures SHALL retain integration exit `2`.

#### Scenario: Standards policy-evaluation owner path reduces a fifth isolated identity
r[octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation]
- GIVEN the reviewed canonical inventory contains 1,110 `non_trait_imports` occurrences represented by 368 stable identities, including one unique private `PolicyEvaluation` import in `crates/octet-standards/src/runner.rs`
- WHEN the import is removed and the pure runner uses explicit `crate::report::PolicyEvaluation` owner paths in the helper return type and construction
- THEN the canonical inventory SHALL contain 2,665 total warning occurrences, 1,815 unique stable identities, and 1,109 `non_trait_imports` occurrences.
- AND selected-policy filtering, registry ordering, enforcement metadata, failure classification, report assembly, positive serialization, and negative blocking-diagnostic behavior SHALL remain unchanged.
- AND the change SHALL NOT add an alias, lint suppression, exception, policy change, or unrelated behavior change.

#### Scenario: Old baseline proves the standards-policy-evaluation removal before refresh
r[octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation.old_baseline]
- GIVEN the owner-qualified standards runner and the previously accepted 1,816-identity baseline
- WHEN the canonical self-check evaluates current findings before baseline replacement
- THEN it SHALL exit `0` with warning-only status, exactly one stale ID, and zero new IDs.
- AND the stale ID SHALL be the reviewed `non_trait_imports` identity `b3:c0f0bf1de2e13e66`.

#### Scenario: Fifth reviewed refresh contains removals only
r[octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation.refresh]
- GIVEN the old-baseline standards-policy-evaluation reduction result is accepted for review
- WHEN the runner writes the replacement baseline
- THEN its stable-ID set SHALL equal the old set minus exactly `b3:c0f0bf1de2e13e66` and SHALL contain no additions.
- AND the unchanged self-check SHALL report zero new IDs and zero stale IDs against the replacement.

#### Scenario: Standards-policy-evaluation regression semantics remain unchanged
r[octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation.regression]
- GIVEN the refreshed lower standards-policy-evaluation baseline and the hermetic one-warning source mutation
- WHEN focused policy-selection and report behavior, complete standards tests, and permanent positive and negative self-check cases run
- THEN selected and unselected policy behavior plus report serialization SHALL pass, accepted warning-only source SHALL exit `0`, and the one-new-ID mutation SHALL exit `1` with exactly one new stable identity.
- AND malformed metadata and Cargo-child failures SHALL retain integration exit `2`.

#### Scenario: Baseline evidence remains bounded
r[octet.runner_self_check.warning_budget.boundary]
r[octet.runner_self_check.warning_budget.reduction.boundary]
r[octet.runner_self_check.warning_budget.reduction.public_unsafe_abi.boundary]
r[octet.runner_self_check.warning_budget.reduction.standards_resource.boundary]
r[octet.runner_self_check.warning_budget.reduction.standards_report.boundary]
r[octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation.boundary]
- GIVEN the monotonic baseline refresh and positive/negative gate pass
- WHEN contributors review self-check status
- THEN the result SHALL establish only removal and membership of stable IDs for the pinned configured static-lint run.
- AND it SHALL NOT claim warning correctness, policy-evaluation completeness, report completeness, standards-check completeness, lint correctness, test or binary coverage, dependency behavior, runtime safety, whole-program correctness, lint completeness, or release eligibility.
