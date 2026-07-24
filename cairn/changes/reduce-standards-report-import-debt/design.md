# Design

## Success Contract

Goal: remove the unique standards-runner `CheckReport` `non_trait_imports` identity through explicit owner paths while preserving pure report assembly, field population, diagnostic ordering, and serialization behavior.

Observable completion evidence:

- `crates/octet-standards/src/runner.rs` has no private `CheckReport` import and uses `crate::report::CheckReport` in the public return type and report construction;
- the accepted 1,817-ID baseline exits `0` with warning-only status, exactly stale ID `b3:b918c07b0ac7abe7`, and zero new IDs;
- the runner-generated candidate is exactly the old stable-ID set minus that identity and contains no additions;
- canonical counts decrease from 2,667 to 2,666 warning occurrences, from 1,817 to 1,816 total stable identities, and from 1,111 to 1,110 `non_trait_imports` occurrences;
- positive report construction/serialization and negative blocking-diagnostic tests pass without lint-policy, exception, suppression, alias, field, ordering, or behavior changes;
- permanent self-check, policy, traceability, source-derived fixture, formatting, Clippy, Nextest, and broad compatible-system gates pass.

False completion includes accepting any new stable ID, lowering only aggregate occurrence counts, deleting the baseline record before observing it stale, changing the public return type or report fields, changing diagnostic order or serialized output, adding an alias/suppression/exception, weakening lint policy, admitting unrelated cleanup, or treating stable-ID membership as proof of warning correctness.

## Approach Registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| standards report type | Remove the unique `CheckReport` import and qualify its return-type and construction uses | validated candidate | Old baseline exits `0` with 2,666 warnings, zero new IDs, and exactly stale ID `b3:b918c07b0ac7abe7` |
| standards policy-evaluation type | Qualify `PolicyEvaluation` in the helper return type and construction | independent/deferred | Similar owner-path mechanism, but the selected report type is already bounded by direct report construction and serialization tests |
| numeric-lint AST imports | Qualify `Local` or `PatKind` in `src/naming/numeric_units.rs` | independent/deferred | Editing lint traversal syntax has a larger semantic audit surface than a report DTO path |
| another private leaf call | Select a single-use import in another import-only owner file | independent/deferred | No redirect is justified unless the report path fails old-baseline or behavior validation |
| alias, exception, or source allow | Hide or admit the import | rejected | Evades the owner-path rule or weakens policy rather than reducing debt |
| broad cohort rewrite | Qualify multiple imports at once | rejected | Enlarges review and incidental identity risk without improving the one-ID completion contract |

Search budget: one structured inventory, four mechanism families, at most three old-baseline executions, two redirect rounds, zero advisory models, and repository validators as authority. Search stops at a candidate surviving deterministic old-baseline and adversarial report-behavior audit.

## Functional Core and Imperative Shell

`run_checks` remains a pure deterministic orchestration core over `CheckInput`; only the source spelling of its report type changes. The same diagnostics, suppressions, evaluated policy IDs, policy evaluations, and capability reports populate the same DTO in the same order. CLI filesystem, process, and rendering effects remain in the existing shell. Existing positive report serialization and negative blocking-diagnostic tests exercise the core without mocks.

The existing pure stable-ID set comparison remains baseline admission authority. The old baseline is immutable during observation; only a runner-generated candidate with no additions and the exact reviewed removal may replace it.

## Validation Sequence

1. Preserve the complete standards test result, focused positive/negative report result, and canonical inventory before editing.
2. Apply the owner-qualified report paths and rerun focused report tests.
3. Require the old baseline to report warning-only exit `0`, no new IDs, and exactly stale ID `b3:b918c07b0ac7abe7`.
4. Generate and compare a candidate baseline before replacement; update exact documentation and Nix assertions only after removals-only proof.
5. Refresh source-derived evidence only through the repository-owned fixture generator, then run permanent and broad gates.

## Risks and Claim Boundary

The public `run_checks` signature must remain semantically identical, and report fields must remain present in the same source order. Positive and negative tests constrain serialization and blocking-diagnostic behavior, while complete standards tests constrain orchestration. A source edit can invalidate source-token receipts even when behavior is unchanged; freshness gates remain authoritative.

The result proves only that one reviewed stable warning identity is absent from the pinned configured workspace-library run and no unreviewed identity entered its replacement baseline. It does not prove warning correctness, report completeness, standards-check completeness, test/binary coverage, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.
