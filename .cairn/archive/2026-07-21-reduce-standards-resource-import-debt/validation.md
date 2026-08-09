# Validation

## Outcome

Octet's third post-ratchet warning-debt reduction removes the unique `non_trait_imports` stable identity for the private `check_resources` import in `crates/octet-standards/src/runner.rs`. The pure `run_checks` orchestration core now calls `crate::resource::check_resources` explicitly at the same point after capability diagnostics and before verified-proof diagnostics, without aliases, suppressions, exception changes, lint-policy changes, reordered checks, or behavior changes.

The canonical inventory decreased from 2,668 to 2,667 warning occurrences, from 1,818 to 1,817 unique stable identities, and from 1,112 to 1,111 `non_trait_imports` occurrences. `negated_predicate` remains zero. The replacement baseline BLAKE3 identity is `b3:10c3f2decf773236c05abc465f1bc1b264933cba929a90133abc8f31502e9546`.

The unchanged self-check exits `0` with `warning-only` status, zero errors, zero new IDs, and zero stale IDs. The permanent behavior-preserving negative mutation still exits `1` with exactly one new `negated_predicate` identity. Malformed metadata and Cargo-child failures remain integration exit `2` cases.

Current Cairn traceability policy hash is `73d0b853284ca1aa2a1a743759c9229a5e84ba2d1e0a092e1f64808676ffe346`. Traceability reports `missing: []` and `dangling: []`.

## Portfolio audit

The bounded search inspected 370 remaining `non_trait_imports` identities across 1,112 occurrences. The selected runner file had seven stable identities and ten occurrences, all from `non_trait_imports`; the `check_resources` identity occurred once. Completion required old-baseline removals with no additions.

| Family | Result | Evidence |
| --- | --- | --- |
| standards resource call | validated | Qualifying the one `check_resources` call produced 2,667 warnings, zero new IDs, and exactly stale ID `b3:3965b4dd27194e41` |
| standards report type | deferred | The owner file is import-only, but the resource call has direct positive bounded and negative unbounded behavior tests |
| numeric-lint AST imports | deferred | Qualifying `Local` or `PatKind` changes lint traversal syntax and has a larger semantic audit surface |
| another private leaf call | deferred | No redirect was justified after the resource path survived deterministic old-baseline and behavior validation |
| aliases, exceptions, suppressions, policy changes, broad rewrites | rejected | These retain or hide accepted debt, weaken policy, or enlarge unrelated identity and review risk |

The surviving candidate's adversarial audit confirmed that the resource check remains present at the same ordering boundary, bounded-channel acceptance and unbounded-channel rejection still pass, the import identity occurs once, the owner file has no non-import finding identity that can churn, no unrelated stable ID changed, and the runner-generated baseline preserves current review context.

Declared budget was one inventory pass, four mechanism families, at most three old-baseline runs, two redirect rounds, and zero advisory models. Used budget was one inventory pass, four families, three old-baseline runs, zero redirect rounds, and zero advisory models. The third run followed repository formatting and was the final allowed audit. Search stopped on deterministic validation.

## Baseline-set evidence

The runner-generated candidate was compared as a stable-ID set before installation:

- old count: 1,818;
- candidate count: 1,817;
- additions: `[]`;
- removals: `["b3:3965b4dd27194e41"]`;
- old hash: `b3:23efb13653cef018dd76dc03eb6319bbd1946c67089add0e53db4c8c673b357e`;
- candidate hash: `b3:10c3f2decf773236c05abc465f1bc1b264933cba929a90133abc8f31502e9546`.

The final post-format old-baseline audit again reports 2,667 findings, 1,817 current/accepted-current identities, zero new IDs, and exactly stale ID `b3:3965b4dd27194e41`. The refreshed canonical run reports 1,817 accepted/current identities, 1,111 `non_trait_imports` occurrences, zero removed-identity occurrences, zero `negated_predicate` occurrences, and zero new/stale IDs.

## Checks

- Pre-change canonical run — exit `0`, `warning-only`, 2,668 warnings; complete standards baseline passed with 127 library tests and 24 CLI tests.
- Focused post-change resource behavior tests — positive bounded and negative unbounded cases both passed.
- Three old-baseline audits — each admissible final source shape exited `0` with 2,667 warnings, zero new IDs, and exactly stale ID `b3:3965b4dd27194e41`; the final audit followed rustfmt.
- Runner-generated candidate set comparison — exact one-ID removal, no additions, current review context.
- Refreshed canonical structured run — exit `0`, `warning-only`, 2,667 warnings, 1,817 accepted/current IDs, zero errors, zero new/stale IDs.
- Complete post-format standards tests — 127 library and 24 CLI tests passed.
- Hermetic `cargo-octet-self-check` Nix gate — refreshed positive case, exactly-one-new-ID mutation, malformed metadata, and Cargo-child failure contracts passed.
- Repo-owned object-corpus refresh test — 1 passed and refreshed only the expanded corpus object-set BLAKE3 after the standards source edit and rustfmt.
- Focused object-corpus tests — 12 passed; fixture freshness and independent fresh-build determinism Nix gates passed.
- Cairn policy export freshness — passed.
- Strict Cairn validation — passed with no issues.
- Cairn traceability — `missing: []`, `dangling: []`.
- Permanent positive/negative `cairn-traceability` Nix gate — passed.
- `cargo fmt --all -- --check` — initially exposed a line-wrap mismatch; rustfmt was applied, the source-derived receipt was regenerated, and the final check passed.
- Workspace/all-target Clippy with warnings denied — passed.
- Nextest — 232 passed, 2 configured tests skipped.
- Full compatible-system `nix flake check -L --option secret-key-files ''` — passed.

## Functional core and shell

`run_checks` remains a pure deterministic core over supplied `CheckInput`; only the source spelling of one module-owned call changed. The resource checker stays between the same capability and proof-check stages. The existing CLI remains the imperative shell for file, environment, and process effects. Positive bounded-channel and negative unbounded-channel tests exercise the resource behavior without mocks.

The existing pure stable-ID set comparison remained the only baseline admission authority.

## Non-claims

The result proves only that one reviewed stable warning identity was removed from the pinned workspace-library static-lint run and that no unreviewed identity entered the refreshed baseline. It does not prove the remaining warnings correct, low-noise, harmless, or complete. It does not prove resource-analysis completeness, standards-check completeness, lint correctness, test/binary/example coverage, dependency behavior, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.

Traceability proves selected-policy linkage only, not runtime reachability or requirement truth. Nix completed locally after the known unavailable Git HTTPS helper and configured remote-builder warnings.
