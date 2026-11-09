## Why

Octet already separates verified logic from implementation. Proof receipts are scoped to a recorded model, verifier, and stated assumptions, the evidence tiers keep `formal_proof` distinct from `static_lint` and `manual_review_required`, and the trust-boundary map says that a clean run is an input to review rather than a complete assurance argument.

What the repository does not yet have is a name for the defect that lives in the space those two statements describe. When a verified model's assumptions do not hold in the running system, every model-level proof still passes. Four specification assumptions cover most of that space: the synchronous process assumption, the request-to-response correlation assumption, atomic persistence of related protocol state, and formal coverage of the entire protocol, including extensions.

A reviewer who finds an interleaving defect, a torn-write defect, or an extension that no proof covers currently has no shared vocabulary for the finding and no named place to look for the evidence that would close it. The result is the same failure the evidence tiers already warn about: a verified model is read as a verified implementation.

## What Changes

- Add a documented failure class named for the proof-to-implementation gap, with the four specification assumptions named verbatim.
- State plainly that a verified model is not a verified implementation.
- Name the three layers the class lives in, interleaving, fault handling, and unverified extensions, and name the kind of system-level testing that closes each layer.
- State the reporting shape for a defect in the class, so the finding names the violated assumption and the layer instead of asserting that a proof is wrong.
- Link the class from the repository README, the trust-boundary evidence index, the contributor guide, the law-tested deterministic cores guide, and the standards verification chapter.
- Bind the class to the accepted `law-tested-deterministic-cores` capability, so the property-based testing doctrine states that law and property tests over a pure core do not close these layers.

## Impact

- **Files**: `docs/proof-implementation-gap.md` (new), `docs/law-tested-cores.md`, `docs/trust-boundary-evidence.md`, `docs/contributor-guide.md`, `docs/standards/source/verification.adoc`, `README.md`, `.cairn/specs/proof-implementation-gap/spec.md` (new, synced from this change), `.cairn/specs/law-tested-deterministic-cores/spec.md` (modified, synced from this change).
- **Testing**: documentation-only change. Deterministic checks are the Cairn proposal, design, and tasks gates, `cairn validate --root . --strict`, the documentation link and wording checks in the verification coverage table, and the repository documentation gates that read `README.md` and `docs/`.
- **Boundary**: this change documents a failure class. It does not claim that any existing proof is unsound, it does not claim that the system-level testing which closes the three layers exists in this repository, and it does not add a checker, a lint, or an evidence rail.
- **Ordering**: the new documentation must land with the specification delta, because the accepted capability names the documentation surfaces that carry the class.

## Out of Scope

- No system-level interleaving, fault-injection, or extension-conformance test campaign. The closing evidence named for each layer is an open obligation, not a delivered rail.
- No change to proof receipts, evidence tiers, or `cargo octet` behavior. The class is a documentation and vocabulary artifact.
- No claim about a specific consumer repository, project, or upstream proof. The class is stated generically.
- No Antithesis capability, coverage, or parity claim.
- No change to the lint catalog, the standards policy registry, or the Nickel policy surface.
