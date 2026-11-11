# Proposal: Wire the measured Vermilion canary as recorded-only evidence

## Why

The live Vermilion u64 pilot produced a measured canary receipt, but the canary
schema is not the typed `octet-vermilion-proof-receipt/v1` contract. The typed
positive fixture is synthetic and uses placeholder artifact identities. Wiring
the canary as formal proof would create a false measured claim.

The canary can still be admitted as recorded-only evidence with a manual-review
proof card that preserves its exact bytes, pinned toolchain, and non-claims.

## What Changes

- Add a closed canary receipt validator for the measured Vermilion u64 pilot.
- Add a `vermilion-canary` proof rail and advisory evidence policy.
- Make `cargo octet prove` validate the canary and emit a
  `manual_review_required` proof card with no formal-proof envelope.
- Preserve the recorded-only tier through artifact verification and Valence
  export.
- Add positive, tamper, drift, and integration tests.

## Non-claims

The canary is not a typed formal receipt, not a fresh provider execution, and
not a formal proof. It does not prove translation soundness, provider
soundness, whole-program correctness, semantic equivalence, or release
eligibility. It does not promote the synthetic typed fixture.
