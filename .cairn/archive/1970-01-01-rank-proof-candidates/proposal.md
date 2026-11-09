# Proposal: Rank proof candidates from Tiger Style allowances

## Why

Every suppressed Tiger Style lint is an unproven claim. Octet already has a
precise signal for the highest-value proof work: an allowance or waiver that has
no validated `proof-ref`. Today an operator has to find those sites by hand, and
there is no bounded path from the allowance to a linked proof card or a test of
whether the linked rail is about the intended property.

## What Changes

- Add `cargo octet prove --candidates [--json]` over the checked source.
- List suppressions that have no validated proof-ref, grouped by lint, with the
  existing rail mapped to that lint when one exists.
- Rank candidates deterministically by lint severity and by trust-boundary
  membership read from the fenced prefix list in
  `docs/trust-boundary-evidence.md`.
- Report every unmapped lint explicitly as unsupported instead of implying a
  proof rail exists.
- Add `cargo octet prove --record-negative-check` to compare a baseline
  proof-card deck with a mutant-run deck and record the result in the card.
- Add `docs/skills/prove-a-candidate/SKILL.md` and a fixture-crate walkthrough
  transcript.
- Add paired positive and negative tests for candidate listing, drop-off after a
  validated proof-ref, stable JSON ordering, unsupported reporting, and negative
  check outcomes.

## Impact

- **CLI**: new opt-in `prove` modes; starter `cargo octet check` is unchanged.
- **Evidence**: proof cards gain an optional `negative_check` record.
- **Docs**: trust-boundary prefixes become a machine-read ranking input.
- **Fixtures**: a standalone proof-candidate fixture crate and a checked-in
  negative-check proof-card deck.

## Out of Scope

- Do not add a new verifier, theorem, or proof rail for narrowing, overflow, or
  indexing properties.
- Do not claim that a linked addition-bound rail proves a narrowing property.
- Do not scan or rank non-Tiger Style lints such as Clippy or compiler allows.
- Do not change starter check behavior, lint registration, or existing rail
  identities.
- Do not treat a mutant failure as general theorem-to-code equivalence.
