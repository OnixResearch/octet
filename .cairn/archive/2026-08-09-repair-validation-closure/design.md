## Context

The checked-in baseline accepted 1,815 stable identities and 2,665 finding occurrences. The first packaged run reported 1,857 identities: 1,800 accepted, 57 new, and 15 stale. Thirteen stale identities had direct semantic replacements, two old imports were resolved, and 44 candidate identities required source repair. The Nix derivation previously stopped at the first nonzero command without printing this evidence.

The final candidate has 1,813 stable identities and 2,660 occurrences. It contains 13 exact semantic rekeys, two reviewed removals, no unpaired candidate identities, and no semantic occurrence growth.

The catalog freshness test independently reported two README contract failures: the total marker did not match `Dylint lints`, and canonical namespace prose did not contain `tigerstyle::<domain>`. Isolated workspace validation also exposed stale primary links and claim-boundary wording in documentation tests.

## Decisions

### Decision: Repair source before baseline refresh

**Choice:** Remove every unapproved candidate identity in source. Refresh the baseline only after an exact semantic comparison proves that remaining additions replace reviewed stale identities.

**Rationale:** A direct baseline regeneration would accept new warning debt and violate the monotonic warning-budget contract.

### Decision: Check identity membership and occurrence multiplicity

**Choice:** Compare stable identity sets and normalized semantic occurrence counts against the last accepted run.

**Rationale:** Stable IDs can repeat across crate contexts. Identity membership alone can hide added warning occurrences.

### Decision: Keep failure output bounded

**Choice:** Print baseline counts, stale IDs, and candidate finding summaries when structured artifacts exist. Otherwise print only the human summary or a bounded stderr tail.

**Rationale:** Operators need the failing evidence without multi-megabyte Nix logs.

### Decision: Preserve public module paths during decomposition

**Choice:** Move oversized nominal-domain implementation files behind Rust `#[path]` modules and compatibility re-exports where needed.

**Rationale:** Internal decomposition must not require downstream import changes.

## Risks / Trade-offs

- Source decomposition can change checked source identities and requires corpus freshness validation.
- Baseline identities can change again after source repairs, so the final migration uses the final candidate output.
- Occurrence comparison requires the accepted run output from revision `9188039e`, not only its deduplicated baseline.
- Passing the self-check proves only the pinned static-lint run and warning membership contract.
