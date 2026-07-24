# Design

## Success Contract

Goal: remove the unique standards-runner `PolicyEvaluation` `non_trait_imports` identity through explicit owner paths while preserving pure policy selection, policy-evaluation construction, ordering, enforcement metadata, failure classification, report assembly, and serialization behavior.

Observable completion evidence:

- `crates/octet-standards/src/runner.rs` has no private `PolicyEvaluation` import and uses `crate::report::PolicyEvaluation` in the helper return type and construction;
- the accepted 1,816-ID baseline exits `0` with warning-only status, exactly stale ID `b3:c0f0bf1de2e13e66`, and zero new IDs;
- the runner-generated candidate is exactly the old stable-ID set minus that identity and contains no additions;
- canonical counts decrease from 2,666 to 2,665 warning occurrences, from 1,816 to 1,815 total stable identities, and from 1,110 to 1,109 `non_trait_imports` occurrences;
- selected-policy and unselected-policy behavior, policy-evaluation fields and order, positive report serialization, and negative blocking-diagnostic behavior remain unchanged;
- permanent self-check, policy, traceability, source-derived fixture, formatting, Clippy, Nextest, and broad compatible-system gates pass.

False completion includes accepting any new stable ID, lowering only aggregate occurrence counts, deleting the baseline record before observing it stale, changing policy filtering or ordering, changing `current_enforcement_mode`, `target_enforcement_mode`, or `failure_type`, changing report fields or serialized output, adding an alias/suppression/exception, weakening lint policy, admitting unrelated cleanup, or treating stable-ID membership as proof of warning correctness.

## Approach Registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| standards policy-evaluation type | Remove the unique `PolicyEvaluation` import and qualify its helper return-type and construction uses | validated candidate | Old baseline exits `0` with 2,665 warnings, zero new IDs, and exactly stale ID `b3:c0f0bf1de2e13e66` |
| standards source-file type | Qualify `SourceFile` in runner inputs and helper signatures | independent/deferred | More use sites and public input spelling enlarge the source and review surface |
| standards checker functions | Qualify another imported checker call | independent/deferred | Alters orchestration spelling at behavior-sensitive call points without need if the DTO path validates |
| numeric-lint AST imports | Qualify `Local` or `PatKind` in `src/naming/numeric_units.rs` | independent/deferred | Editing lint traversal syntax has a larger semantic audit surface than a report DTO path |
| alias, exception, or source allow | Hide or admit the import | rejected | Evades the owner-path rule or weakens policy rather than reducing debt |
| broad cohort rewrite | Qualify multiple imports at once | rejected | Enlarges review and incidental identity risk without improving the one-ID completion contract |

Search budget: one structured inventory, four live mechanism families, at most three old-baseline executions, two redirect rounds, zero advisory models, and repository validators as authority. Search stops at a candidate surviving deterministic old-baseline and adversarial policy-evaluation audit.

## Functional Core and Imperative Shell

`run_checks`, `evaluated_policy_ids`, `policy_evaluations`, and `policy_failure_type` remain pure deterministic cores over supplied registry and selection values; only the source spelling of the policy-evaluation DTO changes. The same selected policies produce the same ordered `PolicyEvaluation` values and the same report. CLI filesystem, process, rendering, and exit effects remain in the existing shell. Existing selected/unselected policy, serialization, and blocking-diagnostic tests exercise these cores without mocks.

The existing pure stable-ID set comparison remains baseline admission authority. The old baseline is immutable during observation; only a runner-generated candidate with no additions and the exact reviewed removal may replace it.

## Validation Sequence

1. Preserve the complete standards test result, focused selected/unselected policy and report results, and canonical inventory before editing.
2. Apply the owner-qualified `PolicyEvaluation` paths and rerun focused standards tests.
3. Require the old baseline to report warning-only exit `0`, no new IDs, and exactly stale ID `b3:c0f0bf1de2e13e66`.
4. Generate and compare a candidate baseline before replacement; update exact documentation and Nix assertions only after removals-only proof.
5. Refresh source-derived evidence only through the repository-owned fixture generator, then run permanent and broad gates.

## Risks and Claim Boundary

The helper return type must remain the same Rust type, selected policies must remain in registry order, and every policy-evaluation field must retain its prior value. Focused positive and negative selection/report tests constrain behavior, while complete standards tests constrain orchestration. A source edit can invalidate source-token receipts even when behavior is unchanged; freshness gates remain authoritative.

The result proves only that one reviewed stable warning identity is absent from the pinned configured workspace-library run and no unreviewed identity entered its replacement baseline. It does not prove warning correctness, policy-evaluation completeness, standards-check completeness, test/binary coverage, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.
