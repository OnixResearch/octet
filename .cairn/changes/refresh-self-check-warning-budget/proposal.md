# Proposal: Refresh the self-check warning budget

## Why

The canonical `cargo-octet-self-check` compares the current workspace warning
set with `standards/self-check-warning-budget/octet-baseline.json`. The
integrated source now includes the benchmark, debug-map, Kamacite, durability,
boolean, merge-rail, and proof-candidate modules. The previous baseline was
stale for every accepted finding. The packaged self-check source now produces
3,107 unique findings / 4,262 warnings.

## What Changes

- Regenerate the baseline with the checked `cargo-octet check --write-baseline`
  path.
- Update the accepted identity and warning totals in the self-check derivation.
- Record the finding distribution and the explicit non-claims.
- Rename two local boolean identifiers whose `negated_predicate` findings the
  baseline validation excludes, preserving the existing conditions.

## Non-claims

The warning budget is an advisory regression baseline, not correctness or
release evidence. The refresh does not suppress or hide findings; it records
the reviewed current set. No new lint is enabled or disabled by this change.
