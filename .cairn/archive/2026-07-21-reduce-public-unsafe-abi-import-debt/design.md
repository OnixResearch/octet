# Design

## Success contract

Goal: remove the unique `ExternAbi` `non_trait_imports` identity from Octet's canonical self-check through an explicit owner path while preserving public-unsafe-API classification behavior.

Observable completion evidence:

- `src/safety/public_unsafe_api.rs` has no private `ExternAbi` import and matches Rust ABI with `rustc_abi::ExternAbi::Rust`;
- the accepted 1,819-ID baseline exits `0` with warning-only status, exactly stale ID `b3:672b1993827f233b`, and zero new IDs;
- the runner-generated candidate is exactly the old stable-ID set minus that identity and contains no additions;
- canonical counts decrease from 2,669 to 2,668 warning occurrences, from 1,819 to 1,818 total stable identities, and from 1,113 to 1,112 `non_trait_imports` occurrences;
- positive Rust-ABI detection and negative extern-ABI exclusion tests pass without lint-policy, exception, suppression, alias, or behavior changes;
- permanent self-check, policy, traceability, source-derived fixture, formatting, Clippy, Nextest, and broad compatible-system gates pass.

False completion includes accepting any new stable ID, lowering only aggregate occurrence counts, deleting the baseline record before observing it stale, changing ABI classification, adding an alias/suppression/exception, weakening lint policy, admitting unrelated cleanup, or treating stable-ID membership as proof of warning correctness.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| root configuration import | Qualify the unique `Config` import in `src/lib.rs` | falsified | Old-baseline run produced 2 new and 3 stale IDs because two whole-file diagnostics changed identity along with the target import |
| leaf ABI classification | Remove the unique `ExternAbi` import and qualify its one enum-variant use | validated candidate | Old baseline exits `0` with 2,668 warnings, zero new IDs, and exactly stale ID `b3:672b1993827f233b` |
| numeric-lint AST imports | Qualify `Local` or `PatKind` in `src/naming/numeric_units.rs` | independent/deferred | The file has only import findings, but editing lint traversal syntax has a larger semantic audit surface than one ABI enum path |
| standards runner calls | Qualify one unique checker import in `crates/octet-standards/src/runner.rs` | independent/deferred | The file has only import findings, but the selected ABI candidate has one use and stronger focused positive/negative tests |
| alias, exception, or source allow | Hide or admit the import | rejected | Evades the owner-path rule or weakens policy rather than reducing debt |
| broad cohort rewrite | Qualify multiple imports at once | rejected | Enlarges review and incidental identity risk without improving the one-ID completion contract |

Search budget: one structured inventory, four mechanism families, at most four old-baseline executions, two redirect rounds, zero advisory models, and repository validators as authority. Search stops at a candidate surviving deterministic old-baseline and adversarial behavior audit.

## Functional core and imperative shell

`signature_uses_rust_abi` remains a pure deterministic predicate over `rustc_hir::FnSig`; only the path spelling of `ExternAbi::Rust` changes. The surrounding lint pass remains the imperative rustc callback shell. Existing positive classification and negative extern-ABI tests exercise the pure classification facts without mocks.

The existing pure stable-ID set comparison remains baseline admission authority. The old baseline is immutable during observation; only a runner-generated candidate with no additions and the exact reviewed removal may replace it.

## Validation sequence

1. Preserve focused public-unsafe-API, numeric-lint, standards-runner, and canonical baselines.
2. Apply the owner-qualified ABI path and run focused positive/negative tests.
3. Require the old baseline to report warning-only exit `0`, no new IDs, and exactly stale ID `b3:672b1993827f233b`.
4. Generate and compare a candidate baseline before replacement; update exact documentation and Nix assertions only after removals-only proof.
5. Refresh source-derived evidence only through the repository-owned fixture generator, then run permanent and broad gates.

## Risks and claim boundary

The path spelling must still classify `ExternAbi::Rust` as Rust and every other ABI as non-Rust. Positive and negative unit/UI cases constrain that boundary. A source edit can invalidate source-token receipts even when behavior is unchanged; freshness gates remain authoritative.

The result proves only that one reviewed stable warning identity is absent from the pinned configured workspace-library run and no unreviewed identity entered its replacement baseline. It does not prove warning correctness, ABI-model completeness, lint completeness, test/binary coverage, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.
