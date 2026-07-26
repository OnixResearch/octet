# Proposal: Generalize the Aeneas consumer contract

## Why

Octet ships a pinned Aeneas, Charon, and Lean provider with one fixture-backed proof rail. The provider proves its execution and evidence path, but it does not publish a reusable downstream consumer contract.

Trellis now has an active exact-source pilot for `src/arithmetic.rs::abs_diff`. That pilot must consume Octet policy without rebuilding provider pins or using an Octet fixture as shadow source.

A typed consumer contract will separate fixed provider authority from consumer-owned source selection, theorem source, requirement mapping, and local claim ownership.

## What Changes

- Add a closed Nickel contract for downstream Aeneas consumer profiles.
- Bind each profile to the reviewed provider profile and compatibility matrix by exact identity.
- Extend proof-rail metadata with exact selected-item, theorem, requirement, trust-closure, and consumer identities.
- Use the existing `cargo octet prove` policy-artifact and rail-selection surfaces.
- Classify outcomes as `supported`, `blocked`, or `failed` from typed stage observations.
- Add an Octet-owned external-consumer fixture with positive and negative evidence.
- Export verified consumer receipts through the existing Valence Aeneas chain boundary.
- Document the staged AeneasVerif roadmap and the deferred Eurydice and Scylla boundaries.

## Impact

- **Files**: Aeneas proof-rail contracts, generated policy artifacts, proof-rail DTOs, execution planning, fixtures, documentation, and this Cairn change.
- **Compatibility**: The shipped fixture rail, Verus rails, starter checks, and provider closure remain valid.
- **Testing**: Nickel fixtures, pure validation tests, bounded execution tests, artifact tamper tests, provider compatibility checks, and Cairn gates.

## Non-goals

- Do not add another proof-assistant backend.
- Do not add Eurydice or Scylla execution.
- Do not make Octet discover a live sibling repository.
- Do not make raw LLBC a stable stack-wide API.
- Do not claim translation soundness, semantic equivalence, verifier soundness, whole-program correctness, or release eligibility.
