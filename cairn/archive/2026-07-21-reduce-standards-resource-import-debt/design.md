# Design

## Success contract

Goal: remove the unique standards-runner `check_resources` `non_trait_imports` identity through an explicit owner path while preserving pure runner behavior, resource-check ordering, and diagnostics.

Observable completion evidence:

- `crates/octet-standards/src/runner.rs` has no private `check_resources` import and calls `crate::resource::check_resources` at the same orchestration point;
- the accepted 1,818-ID baseline exits `0` with warning-only status, exactly stale ID `b3:3965b4dd27194e41`, and zero new IDs;
- the runner-generated candidate is exactly the old stable-ID set minus that identity and contains no additions;
- canonical counts decrease from 2,668 to 2,667 warning occurrences, from 1,818 to 1,817 total stable identities, and from 1,112 to 1,111 `non_trait_imports` occurrences;
- positive bounded-resource acceptance and negative unbounded-resource rejection tests pass without lint-policy, exception, suppression, alias, behavior, or diagnostic-order changes;
- permanent self-check, policy, traceability, source-derived fixture, formatting, Clippy, Nextest, and broad compatible-system gates pass.

False completion includes accepting any new stable ID, lowering only aggregate occurrence counts, deleting the baseline record before observing it stale, reordering or omitting the resource check, changing diagnostics, adding an alias/suppression/exception, weakening lint policy, admitting unrelated cleanup, or treating stable-ID membership as proof of warning correctness.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| standards resource call | Remove the unique `check_resources` import and qualify its one pure-runner call | validated candidate | Old baseline exits `0` with 2,667 warnings, zero new IDs, and exactly stale ID `b3:3965b4dd27194e41` |
| standards report type | Qualify one report type in the runner signature or report construction | independent/deferred | The owner file is import-only, but the selected call path has direct positive and negative resource behavior tests |
| numeric-lint AST imports | Qualify `Local` or `PatKind` in `src/naming/numeric_units.rs` | independent/deferred | Editing lint traversal syntax has a larger semantic audit surface than one standards orchestration path |
| another private leaf call | Select a single-use import in another import-only owner file | independent/deferred | No additional candidate run is needed after the resource path survives deterministic validation |
| alias, exception, or source allow | Hide or admit the import | rejected | Evades the owner-path rule or weakens policy rather than reducing debt |
| broad cohort rewrite | Qualify multiple imports at once | rejected | Enlarges review and incidental identity risk without improving the one-ID completion contract |

Search budget: one structured inventory, four mechanism families, at most three old-baseline executions, two redirect rounds, zero advisory models, and repository validators as authority. Search stops at a candidate surviving deterministic old-baseline and adversarial behavior audit.

## Functional core and imperative shell

`run_checks` remains a pure deterministic orchestration core over `CheckInput`; only the source path spelling of `check_resources` changes. The resource checker remains at the same point after capability diagnostics and before verified-proof diagnostics. CLI filesystem and process behavior remain in the existing shell. Positive bounded-channel and negative unbounded-channel tests exercise the resource classification facts without mocks.

The existing pure stable-ID set comparison remains baseline admission authority. The old baseline is immutable during observation; only a runner-generated candidate with no additions and the exact reviewed removal may replace it.

## Validation sequence

1. Preserve the complete standards test result, focused positive/negative resource result, and canonical inventory before editing.
2. Apply the owner-qualified standards resource path and rerun focused resource tests.
3. Require the old baseline to report warning-only exit `0`, no new IDs, and exactly stale ID `b3:3965b4dd27194e41`.
4. Generate and compare a candidate baseline before replacement; update exact documentation and Nix assertions only after removals-only proof.
5. Refresh source-derived evidence only through the repository-owned fixture generator, then run permanent and broad gates.

## Risks and claim boundary

The resource checker must remain present and in the same diagnostic sequence. Positive and negative tests constrain bounded/unbounded resource behavior, while complete standards and structured runner checks constrain orchestration. A source edit can invalidate source-token receipts even when behavior is unchanged; freshness gates remain authoritative.

The result proves only that one reviewed stable warning identity is absent from the pinned configured workspace-library run and no unreviewed identity entered its replacement baseline. It does not prove warning correctness, resource-analysis completeness, standards-check completeness, test/binary coverage, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.
