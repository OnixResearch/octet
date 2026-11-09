## Context

Octet owns the checked path, the evidence tiers, and the property-based testing doctrine for pure deterministic cores. It already states, in the trust-boundary map and in the verification standards chapter, that proof receipts are scoped to a model and that a clean run is an input to review.

The gap this change fills is a vocabulary gap at the proof-to-implementation boundary. The repository can say that a proof does not cover an implementation, but it does not name the specific defect shapes that sit in that space, and it does not point at the kind of testing that closes them.

The change is documentation-only. It adds no code, no checker, no lint, and no evidence rail, and it does not modify proof receipts or evidence tiers.

## Decisions

### Decision: The class gets its own accepted capability

**Choice:** Add a new `proof-implementation-gap` capability instead of extending an existing one, and state the class in `docs/proof-implementation-gap.md`.

**Covers:** `r[octet.proof_implementation_gap.assumptions]`, `r[octet.proof_implementation_gap.layers]`, `r[octet.proof_implementation_gap.claim_boundary]`, `r[octet.proof_implementation_gap.reporting]`

**Rationale:** The class is a cross-cutting defect vocabulary that spans proof rails, evidence tiers, and testing policy. Folding it into an existing capability would hide it inside a narrower subject and would make it harder to find from the repository entry points.

**Alternative considered:** Extend `scope-boundary` only. Rejected, because that capability states what Octet does not claim, not what a defect between a model and an implementation looks like.

### Decision: The three layers are named as interleaving, fault handling, and unverified extensions

**Choice:** Present the class as exactly three layers, each with the specification assumption or assumptions at risk in it and the kind of system-level testing that closes it.

**Covers:** `r[octet.proof_implementation_gap.layers]`

**Rationale:** Three layers keep the vocabulary small enough to be used in a review. Mapping the four assumptions onto them gives every assumption exactly one home: interleaving holds the synchronous process assumption and the request-to-response correlation assumption, fault handling holds atomic persistence of related protocol state, and unverified extensions holds formal coverage of the entire protocol including extensions.

**Alternative considered:** Give each assumption its own layer. Rejected, because interleaving and correlation fail together under the same scheduler, and separating them hides the shared cause.

### Decision: The claim boundary forbids two specific overclaims

**Choice:** Require the documentation to state that a verified model is not a verified implementation, to state that it does not claim any existing proof is unsound, and to state that the system-level testing which closes the three layers does not exist in this repository today.

**Covers:** `r[octet.proof_implementation_gap.claim_boundary]`

**Rationale:** A failure class that names an unsoundness risk can itself become an unsoundness claim, and a class that names closing evidence can be read as a claim that the closing evidence exists. Both readings are wrong here, so both are forbidden in the text.

**Alternative considered:** Leave the claim boundary to reviewer discretion. Rejected, because the failure mode of this change is exactly an overclaim.

### Decision: The property-based testing doctrine links the class

**Choice:** Extend the accepted `law-tested-deterministic-cores` requirement so that law and property tests over a pure core are never presented as implementation evidence, and link the class from `docs/law-tested-cores.md`.

**Covers:** `r[octet.proof_implementation_gap.property_testing_boundary]`, `r[octet.law_tested_deterministic_cores.fixture_complement]`

**Rationale:** Octet owns the property-based testing doctrine, so the doctrine is where a reader is most likely to reach for the wrong conclusion. The layers are properties of a running system's scheduling, failure, and extension behavior, not of a pure function, so the boundary belongs beside the doctrine rather than only in the new page.

**Alternative considered:** Mention the class only in the new page. Rejected, because the new page would then only be reachable from the README and would not intercept the property-testing reasoning it is most likely to correct.

### Decision: The class is documented, not enforced

**Choice:** Record the class in prose with a named reporting shape, and do not add a checker, lint, or policy rule that rejects documents.

**Covers:** `r[octet.proof_implementation_gap.reporting]`

**Rationale:** A documentation checker would imply that the class is machine-detectable, and it is not. The defect is visible only when a reviewer compares a model with a running system.

**Alternative considered:** Add a standards policy rule that fails when a document offers a proof receipt as implementation evidence. Rejected, because a false positive on a wording pattern would block a consumer's honest prose without adding evidence.

## Functional core / imperative shell split

- **Pure core**: the naming of the four assumptions, the mapping from assumption to layer, and the statement of what closes each layer. These are fixed in the accepted capability.
- **Imperative shell**: the prose surfaces that carry the class, namely `docs/proof-implementation-gap.md` and the links that reach it from the README, the trust-boundary index, the contributor guide, the law-tested cores guide, and the standards verification chapter.

## Verification plan

- Deterministic content checks that the failure-class documentation names the four assumptions verbatim, names the three layers, states the model-versus-implementation sentence, and states the two forbidden overclaims.
- Deterministic link checks that the class is reachable from the README, the trust-boundary evidence index, the contributor guide, and the law-tested deterministic cores guide.
- Cairn proposal, design, and tasks gates, `cairn validate --root . --strict`, sync of the delta specs into the accepted capabilities, and the archive step.

## Risks / Trade-offs

- The class names an overclaim risk that readers may misapply to a specific proof. Mitigated by the explicit statement that the class records where a proof's reach ends and does not assert that any proof is unsound.
- The class names closing evidence that does not exist in this repository, which could read as a roadmap promise. Mitigated by stating plainly that the repository does not currently run that testing.
- Documentation can drift from the accepted capability. Mitigated by making the accepted capability the source of truth for the four assumption names, the three layer names, and the claim boundary, and by deriving the prose from it.
- The standards verification chapter is a policy-anchored document. Mitigated by adding a new section with a new anchor and leaving the existing anchors and their policy bindings unchanged.

## Non-Goals

- No system-level fault campaign, no interleaving harness, and no fault-injection tooling.
- No change to `cargo octet`, to proof receipts, to evidence tiers, or to the lint catalog.
- No Antithesis comparison, parity claim, or feature mapping.
- No per-project or per-consumer assessment.
