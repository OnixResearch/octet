# Design: Recorded-only Vermilion canary rail

## Context

The measured u64 pilot receipt is
`standards/proof-rails/vermilion-lean/evidence/2026-11-09-u64-pilot/receipt.json`.
It records the exact source, Vermilion/Verus/Lean/Mathlib/cvc5 pins, two
verified obligations, zero `sorry`, a kernel-checked proof twin, and explicit
non-claims. It is a canary receipt, not the typed receipt contract.

## Decisions

- Validate the canary in `octet-standards::vermilion_canary` against the
  reviewed Vermilion profile, exact source BLAKE3, obligation identifiers,
  outcome, and non-claims.
- Add a `vermilion-canary` rail with `verifier_mode = "recorded"` and
  `receipt_binding = "recorded-only"`.
- In `cargo octet prove`, validate the canary without executing a provider and
  emit a proof card with `evidence_tier = manual_review_required`,
  `status = manual_review_required`, and no `formal_proof_envelope`.
- Record the canary receipt and source as manual-review provenance records so
  `artifact verify` accepts the complete bundle and the Valence export preserves
  the recorded-only tier.
- Keep the typed synthetic fixture separate; it is not used as measured proof.

## Verification

- Canary validator accepts the measured receipt and rejects source, toolchain,
  outcome, and non-claim drift.
- The prove rail validates the measured receipt and emits a manual-review card.
- `artifact verify` accepts the recorded-only bundle.
- Valence export contains the canary card and provenance at
  `manual_review_required`.

## Non-claims

No formal-proof promotion, fresh execution, typed-receipt admission, executable
refinement, provider soundness, whole-program correctness, semantic
equivalence, or release authority follows.
