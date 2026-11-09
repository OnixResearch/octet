# Design: Rank proof candidates from Tiger Style allowances

## Context

Octet already binds proof cards, `ProofEvidenceRef` linkage, and trust-boundary
evidence. The missing pieces are discovery and an honest negative check. The
candidate command must stay read-only: it must not run a verifier, emit
`formal_proof` provenance, or imply that a suppression is correct.

The checked source is supplied with repeatable `--source` paths. When omitted,
the candidate mode scans `.`. Directory traversal is sorted, skips dependency,
archive, and tooling trees, ignores symlinks, and bounds each source file to
4 MiB.

## Decisions

### Decision: Scan Rust attributes without parsing a crate

The scanner skips comments, strings, and character literals, then reads `allow`
and `expect` items from `#[...]` and `#![...]` attributes. It normalizes
`tigerstyle::<lint>` and bare `<lint>` spellings, expands Tiger Style group
attributes to member lints, and ignores non-Tiger Style namespaces and
`reason = "..."` items. A source-text scan keeps the mode independent from a
configured Dylint run and from crate compilation.

### Decision: Only checked-in rails are mapped

The mapping table names `unchecked_narrowing` to
`octet.verus_proof_rails.addition-bound-production#rail-0`, following the README
linkage example. The mapping carries an explicit scope note: that rail proves
the declared addition-bound property, not the narrowing property. Every lint
without a table entry is serialized with `rail: null` and appears in
`unsupported_lints`.

### Decision: Ranking is a total order

`rank_score = severity_weight * 2 + trust_boundary`. Severity weight is 3 for
`deny` or `error`, 2 for `warn`, 1 for `allow`, and 0 for unknown. Ties sort by
lint, path, line, column, and attribute. The JSON report sorts candidates by
score descending and then by those stable keys; `unsupported_lints` sorts by
lint. The report contains no timestamps or absolute working-directory state, so
two runs over the same inputs serialize identically.

### Decision: Trust-boundary membership is machine-read from the guide

`docs/trust-boundary-evidence.md` contains a `trust-boundary-prefixes` fenced
block. A candidate is on a trust boundary when its path equals a listed prefix
or starts with it. Missing or unreadable trust-boundary input is a diagnostic,
not an empty rank input.

### Decision: A validated proof-ref removes the candidate

A reference discharges a candidate only when all of these hold:

1. its `consumer_source_path` resolves to the candidate file;
2. its `consumer_context` names the exact `path:line`;
3. `proof-ref` validation passes against the supplied `proof-cards.json` deck;
4. the saved source BLAKE3 and anchor still match.

A reference that exists but fails any check keeps the candidate listed with
`invalid` or `unbound` status. A reference without a deck is `unvalidated`.
Those states make the report blocking.

### Decision: Negative check compares two proof-card decks

`--record-negative-check` reads a baseline deck and a mutant-run deck, requires
exactly one card for the named proof/property ID in the baseline, and classifies
the mutant card:

- `passed` -> `property_not_demonstrated`, exit 1;
- `failed` -> `property_demonstrated`, exit 0, scoped to this mutation;
- anything else or a missing mutant card -> `inconclusive`, exit 1.

The record is written into the baseline card's optional `negative_check` field
with the mutant deck BLAKE3, mutant source label, diagnostics, and a non-claim.
The command refuses to overwrite the mutant deck.

## Rejected alternatives

- **Reuse the Dylint diagnostic stream.** It would require a configured lint
  run, misses allowances not yet exercised by the compiler, and couples triage to
  a build.
- **Infer a rail from the lint name.** Only one lint has a defensible existing
  rail; automatic inference would claim proof coverage that does not exist.
- **Rank by file order or first-match.** It is not deterministic across source
  roots and hides severity differences.
- **Downgrade a proof card when the mutant passes.** The card remains valid for
  its declared scope. The negative-check field carries the missing-property fact
  without rewriting the receipt.
- **Record negative checks in a side file.** The acceptance contract requires the
  result in `proof-cards.json`, so the card schema is the bounded home.

## Test design

Positive coverage:

- tool-scoped, bare, multiline, and `expect` allowances are found;
- group allowances expand to member lints;
- a valid reference drops the candidate and increments `discharged`;
- a mutant failure records `property_demonstrated`;
- JSON output is byte-stable across repeated runs.

Negative coverage:

- comments, strings, and non-Tiger Style namespaces do not become candidates;
- a stale BLAKE3 or unbound context keeps the candidate and blocks;
- an unmapped lint has no rail and appears in `unsupported_lints`;
- a still-passing mutant records `property_not_demonstrated`;
- a missing mutant card records `inconclusive`;
- the negative-check output cannot overwrite the mutant deck.

The fixture crate keeps the allowance at `src/codec.rs:42` and the mutant removes
the bound assertion. The checked-in card deck records the real
`property_not_demonstrated` result from the existing addition-bound rail.

## Risks

- Source text scanning can miss macro-generated attributes. The command reports
  candidates it can see; it does not claim complete suppression coverage.
- The trust-boundary prefix list is a review signal. Adding a prefix does not
  make the code correct.
- The only mapped rail proves a different property than the fixture's narrowing
  site. The negative check exists precisely to expose that gap.
- A mutant failure may come from an unrelated verifier error. The card non-claim
  limits the conclusion to this mutation.

## Claim boundary

Candidate reports are `static_lint`-adjacent triage evidence. A mapped rail still
proves only its `checked_scope`. `proof-ref` validation is linkage evidence and
does not rerun the verifier. `artifact verify` checks saved bytes and provenance,
not code correctness. `property_demonstrated` is scoped to the mutation;
`property_not_demonstrated` is an explicit refusal to claim the intended
property. No surface claims whole-program correctness, certification,
business-domain validation, semantic equivalence, or full safety.
