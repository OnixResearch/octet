# Validation

## Outcome

Octet's fourth post-ratchet warning-debt reduction removes the unique `non_trait_imports` stable identity for the private `CheckReport` import in `crates/octet-standards/src/runner.rs`. The pure `run_checks` orchestration core now uses `crate::report::CheckReport` explicitly in its public return type and report construction, without changing the Rust type, report fields, field order, diagnostics, suppressions, policy evaluations, capability reports, lint policy, exceptions, or suppression state.

The canonical inventory decreased from 2,667 to 2,666 warning occurrences, from 1,817 to 1,816 unique stable identities, and from 1,111 to 1,110 `non_trait_imports` occurrences. `negated_predicate` remains zero. The replacement baseline BLAKE3 identity is `b3:36da78bf5493cf35f6fdc0ea361fafd21b065fb87fa173ddd3f59f33d6b0acb9`.

The unchanged self-check exits `0` with `warning-only` status, zero errors, zero new IDs, and zero stale IDs. The permanent behavior-preserving negative mutation still exits `1` with exactly one new `negated_predicate` identity. Malformed metadata and Cargo-child failures remain integration exit `2` cases.

Cairn traceability reports 174 of 174 accepted requirements referenced under profile `octet-default`. Policy export is fresh, strict Cairn validation reports no issues, and the archive-ready policy hash is `73d0b853284ca1aa2a1a743759c9229a5e84ba2d1e0a092e1f64808676ffe346`.

## Portfolio Audit

The bounded search inspected the 369 remaining `non_trait_imports` identities across 1,111 occurrences. The selected runner file contained only import findings, and the `CheckReport` identity occurred once. Completion required old-baseline removals with no additions.

| Family | Result | Evidence |
| --- | --- | --- |
| standards report type | validated | Qualifying the return type and construction produced 2,666 warnings, zero new IDs, and exactly stale ID `b3:b918c07b0ac7abe7` |
| standards policy-evaluation type | deferred | Similar owner-path mechanism remains available, but no redirect was justified after the report path passed deterministic validation |
| numeric-lint AST imports | deferred | Qualifying `Local` or `PatKind` changes lint traversal syntax and has a larger semantic audit surface |
| another private leaf call | deferred | No additional candidate run was needed after the report path survived deterministic old-baseline and behavior validation |
| aliases, exceptions, suppressions, policy changes, broad rewrites | rejected | These retain or hide accepted debt, weaken policy, or enlarge unrelated identity and review risk |

The surviving candidate's adversarial audit confirmed that the public return type remains the same Rust type, all report fields remain in the same source order, positive serialization and negative blocking-diagnostic behavior pass, the import identity occurs once, the owner file has no non-import finding identity that can churn, no unrelated stable ID changed, and the runner-generated baseline preserves current review context. The surviving `PolicyEvaluation` record updates only its line locator from 9 to 8 after the preceding import removal.

Declared budget was one inventory pass, four mechanism families, at most three old-baseline executions, two redirect rounds, and zero advisory models. Used budget was one inventory pass, four families, one old-baseline execution, zero redirect rounds, and zero advisory models. Search stopped on deterministic validation.

## Baseline-Set Evidence

The runner-generated candidate was compared as a stable-ID set before installation:

- old count: 1,817;
- candidate count: 1,816;
- additions: `[]`;
- removals: `["b3:b918c07b0ac7abe7"]`;
- old hash: `b3:10c3f2decf773236c05abc465f1bc1b264933cba929a90133abc8f31502e9546`;
- candidate hash: `b3:36da78bf5493cf35f6fdc0ea361fafd21b065fb87fa173ddd3f59f33d6b0acb9`.

The old-baseline audit reports 2,666 findings, 1,816 current/accepted-current identities, zero new IDs, and exactly stale ID `b3:b918c07b0ac7abe7`. The refreshed canonical run reports 1,816 accepted/current identities, 1,110 `non_trait_imports` occurrences, zero removed-identity occurrences, zero `negated_predicate` occurrences, and zero new/stale IDs.

A first local installation attempt exposed that the generic Steel `pi-write-file` helper does not truncate a longer existing file. The malformed trailing bytes were detected by exact diff review before validation, replaced from the untouched generated candidate with a truncating copy, and rechecked. The final checked-in baseline diff contains only the hash update, the reviewed finding removal, and the surviving `PolicyEvaluation` line-locator refresh.

## Checks

- Pre-change complete standards baseline — 127 library tests and 24 CLI tests passed.
- Focused post-change report behavior — positive stable JSON serialization and negative blocking-diagnostic cases passed.
- Old-baseline audit — exit `0`, `warning-only`, 2,666 warnings, zero new IDs, and exactly stale ID `b3:b918c07b0ac7abe7`.
- Runner-generated candidate set comparison — exact one-ID removal, no additions, current review context.
- Refreshed canonical structured run — exit `0`, `warning-only`, 2,666 warnings, 1,816 accepted/current IDs, zero errors, zero new/stale IDs.
- Complete post-change standards tests — 127 library and 24 CLI tests passed.
- Hermetic `cargo-octet-self-check` Nix gate — refreshed positive case, exactly-one-new-ID mutation, malformed metadata, and Cargo-child failure contracts passed.
- Repo-owned object-corpus refresh — changed only the expanded corpus object-set BLAKE3 to `b3:4b61ab7f7c191467bc06945cf4db5de416d44121f2f7023e97a9629d8bf5a1b9` after the standards source edit.
- Focused object-corpus tests — 12 passed; fixture freshness reported 3 passed, and independent fresh-build determinism passed.
- Cairn policy export freshness — passed.
- Strict Cairn validation — passed with no issues.
- Cairn traceability — 174/174 requirements referenced; permanent positive/negative `cairn-traceability` Nix gate passed.
- `cargo fmt --all -- --check` — passed.
- Workspace/all-target Clippy with warnings denied — passed.
- Nextest — 232 passed, 2 configured tests skipped.
- Full compatible-system `nix flake check -L --option secret-key-files ''` — passed with `all checks passed`.

## Functional Core and Shell

`run_checks` remains a pure deterministic core over supplied `CheckInput`; only the source spelling of its report type changed. The same report fields are populated from the same values in the same order. The existing CLI remains the imperative shell for file, environment, rendering, and process effects. Positive serialization and negative blocking-diagnostic tests exercise the report behavior without mocks.

The existing pure stable-ID set comparison remained the only baseline admission authority.

## Non-Claims

The result proves only that one reviewed stable warning identity was removed from the pinned workspace-library static-lint run and that no unreviewed identity entered the refreshed baseline. It does not prove the remaining warnings correct, low-noise, harmless, or complete. It does not prove report completeness, standards-check completeness, lint correctness, test/binary/example coverage, dependency behavior, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.

Traceability proves selected-policy linkage only, not runtime reachability or requirement truth. Nix completed locally after the known unavailable Git HTTPS helper and configured remote-builder warnings.
