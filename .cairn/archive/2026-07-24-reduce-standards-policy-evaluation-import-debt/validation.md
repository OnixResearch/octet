# Validation

## Outcome

Octet's fifth post-ratchet warning-debt reduction removes the unique `non_trait_imports` stable identity for the private `PolicyEvaluation` import in `crates/octet-standards/src/runner.rs`. The pure `policy_evaluations` helper now uses `crate::report::PolicyEvaluation` explicitly in its return type and construction, without changing the Rust type, selected-policy filtering, registry order, enforcement metadata, failure classification, report fields, diagnostics, suppressions, lint policy, exceptions, or suppression state.

The canonical inventory decreased from 2,666 to 2,665 warning occurrences, from 1,816 to 1,815 unique stable identities, and from 1,110 to 1,109 `non_trait_imports` occurrences. `negated_predicate` remains zero. The replacement baseline BLAKE3 identity is `b3:af460a97f17e791476bc357eadc8ee22deb1548d5011a924e7edb94de9f7a7dc`.

The unchanged self-check exits `0` with `warning-only` status, zero errors, zero new IDs, and zero stale IDs. The permanent behavior-preserving negative mutation still exits `1` with exactly one new `negated_predicate` identity. Malformed metadata and Cargo-child failures remain integration exit `2` cases.

Cairn traceability reports 179 of 179 accepted requirements referenced under profile `octet-default`. Policy export is fresh, strict Cairn validation reports no issues, and the archive-ready policy hash is `73d0b853284ca1aa2a1a743759c9229a5e84ba2d1e0a092e1f64808676ffe346`.

## Portfolio Audit

The bounded search inspected the 368 remaining `non_trait_imports` identities across 1,110 occurrences. The selected runner identity occurred once. Completion required old-baseline removals with no additions.

| Family | Result | Evidence |
| --- | --- | --- |
| standards policy-evaluation type | validated | Qualifying the helper return type and construction produced 2,665 warnings, zero new IDs, and exactly stale ID `b3:c0f0bf1de2e13e66` |
| standards source-file type | deferred | More use sites and public input spelling enlarge the source and review surface |
| standards checker functions | deferred | Orchestration call spelling has a larger behavior-sensitive surface than the selected DTO path |
| numeric-lint AST imports | deferred | Qualifying `Local` or `PatKind` changes lint traversal syntax and has a larger semantic audit surface |
| aliases, exceptions, suppressions, policy changes, broad rewrites | rejected | These retain or hide accepted debt, weaken policy, or enlarge unrelated identity and review risk |

The surviving candidate's adversarial audit confirmed that the helper return type remains the same Rust type, selected policies remain in registry order, policy-evaluation fields remain sourced from the same values, positive serialization and negative blocking-diagnostic tests pass, the import identity occurs once, no unrelated stable ID changed, and the runner-generated baseline preserves current review context. A full-record comparison of all 1,815 surviving baseline entries found zero changed common records.

Declared budget was one inventory pass, four live mechanism families, at most three old-baseline executions, two redirect rounds, and zero advisory models. Used budget was one inventory pass, four families, one old-baseline execution, zero redirect rounds, and zero advisory models. Search stopped on deterministic validation.

## Baseline-Set Evidence

The runner-generated candidate was compared as a stable-ID set before installation:

- old count: 1,816;
- candidate count: 1,815;
- additions: `[]`;
- removals: `["b3:c0f0bf1de2e13e66"]`;
- old hash: `b3:36da78bf5493cf35f6fdc0ea361fafd21b065fb87fa173ddd3f59f33d6b0acb9`;
- candidate hash: `b3:af460a97f17e791476bc357eadc8ee22deb1548d5011a924e7edb94de9f7a7dc`.

The old-baseline audit reports 2,665 findings, 1,815 current/accepted-current identities, zero new IDs, and exactly stale ID `b3:c0f0bf1de2e13e66`. The refreshed canonical run reports 1,815 accepted/current identities, 1,109 `non_trait_imports` occurrences, zero removed-identity occurrences, zero `negated_predicate` occurrences, and zero new/stale IDs.

The checked-in baseline diff contains only the hash update and the reviewed finding removal. No surviving baseline record changed its locator or content.

## Checks

- Pre-change complete standards baseline — 127 library tests and 24 CLI tests passed.
- Focused post-change standards behavior — 127 library tests passed, including selected/unselected policy, stable JSON serialization, and blocking-diagnostic cases.
- Old-baseline audit — exit `0`, `warning-only`, 2,665 warnings, zero new IDs, and exactly stale ID `b3:c0f0bf1de2e13e66`.
- Runner-generated candidate set comparison — exact one-ID removal, no additions, zero changed surviving records, current review context.
- Refreshed canonical structured run — exit `0`, `warning-only`, 2,665 warnings, 1,815 accepted/current IDs, zero errors, zero new/stale IDs.
- Complete post-change standards tests — 127 library and 24 CLI tests passed.
- Hermetic `cargo-octet-self-check` Nix gate — refreshed positive case, exactly-one-new-ID mutation, malformed metadata, and Cargo-child failure contracts passed.
- Repo-owned object-corpus refresh — changed only the expanded corpus object-set BLAKE3 to `b3:63920b5b86bf5113c0693407b9e66de4e0caf637ef038ac477b744011e7d5a01` after the standards source edit.
- Object-corpus fixture freshness and independent fresh-build determinism — passed; two producers were byte-stable, fixture parity passed, compared fields were stable, and boundary claims remained false.
- Octet policy validation — passed.
- Strict Cairn validation — passed with no issues.
- Cairn traceability — 179/179 requirements referenced; permanent `cairn-traceability` Nix gate passed.
- `cargo fmt --all -- --check` — passed.
- Workspace/all-target Clippy with warnings denied — passed.
- Nextest — 232 passed, 2 configured tests skipped.
- Full compatible-system `nix flake check -L --option secret-key-files ''` — passed with `all checks passed`.

## Validation Notes

An initial external Cairn invocation omitted Octet's generated policy and therefore selected the generic `cairn-default` profile; that non-authoritative run failed as expected. The authoritative rerun used `cairn-policy/generated/cairn-policy.json`, selected `octet-default`, and passed after the accepted spec was synchronized. Before sync, the five new implementation markers were correctly reported as dangling; after sync, coverage passed at 179/179.

Nix completed locally after the known unavailable Git HTTPS helper and configured remote-builder warnings. Those warnings did not change requested check outcomes.

## Functional Core and Shell

`run_checks`, `evaluated_policy_ids`, `policy_evaluations`, and `policy_failure_type` remain pure deterministic cores over supplied registry and selection values; only the source spelling of the policy-evaluation DTO changed. The same selected policies produce the same ordered values and report. The existing CLI remains the imperative shell for file, environment, rendering, and process effects. Positive selection/serialization and negative exclusion/blocking tests exercise the core without mocks.

The existing pure stable-ID set comparison remained the only baseline admission authority.

## Non-Claims

The result proves only that one reviewed stable warning identity was removed from the pinned workspace-library static-lint run and that no unreviewed identity entered the refreshed baseline. It does not prove the remaining warnings correct, low-noise, harmless, or complete. It does not prove policy-evaluation completeness, report completeness, standards-check completeness, lint correctness, test/binary/example coverage, dependency behavior, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.

Traceability proves selected-policy linkage only, not runtime reachability or requirement truth.
