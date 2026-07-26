# Design: Generalize the Aeneas consumer contract

## Context

The current provider profile fixes Aeneas, Charon, OCaml, Lean, extraction policy, execution limits, and non-claims. The shipped policy registry selects one `real_fixture::safe_add` theorem.

The execution shell already accepts source and theorem paths from a selected proof rail. The missing surface is a typed, versioned contract for non-Octet consumers.

The first consumer is Trellis. It owns the selected runtime source, maintained theorem, requirement mapping, cross-verifier manifest, and local claim.

## Decisions

### Decision: Separate provider and consumer policy

**Choice:** Keep the existing provider profile as Octet-owned authority. Add a consumer profile that references its exact profile and compatibility identities.

The consumer profile contains a consumer ID, workspace-relative source root, source paths, selected items, theorem paths, theorem IDs, requirement IDs, claim classes, approved statement BLAKE3 values, external-model inventory, trust inputs, resource policy, and non-claims.

**Rationale:** Consumers can select local proof subjects without changing toolchain authority or rebuilding provider pins.

### Decision: Reuse the existing prove command

**Choice:** Load consumer profiles through existing policy artifacts and select them with `--rail`. Keep `--source` as the explicit capability-root input.

Do not add an ambient profile search or a sibling-repository lookup.

**Rationale:** The current command already separates policy selection from source-root selection. A second parser creates duplicate authority.

### Decision: Bind exact source and selected items

**Choice:** Measure consumer source bytes and preserve source paths and spans through Charon and `translation.json`. Treat `def_id` as a current-run join key only.

Reject copied source, ambiguous item selection, broad crate selection, source drift, stale generated output, and selected-item mismatch.

**Rationale:** A proof over a shadow implementation creates an unproved equivalence obligation.

### Decision: Keep validation pure

**Choice:** Put profile, selection, outcome, theorem, and receipt decisions in pure functions over loaded values. Keep file access and process execution in `cargo-octet`.

**Rationale:** Consumer admission logic must remain deterministic and testable without Aeneas, Charon, Lean, or filesystem setup.

### Decision: Use three outcome classes

**Choice:** Report `supported` only after the complete selected stage path succeeds. Report `blocked` only for a reviewed unsupported source boundary.

Report `failed` for setup, package, process, environment, artifact, policy, or resource errors.

**Rationale:** A missing tool is not evidence that Aeneas rejects the Rust language surface.

### Decision: Preserve consumer theorem ownership

**Choice:** Keep generated Lean definitions separate from maintained theorem source. Bind each theorem to its consumer requirement, claim class, statement identity, and trust closure.

Octet owns execution and theorem-admission policy. The consumer owns theorem intent and local requirement selection.

**Rationale:** A passing checker result does not establish that the theorem states the intended consumer property.

### Decision: Keep downstream adoption revision-bound

**Choice:** Add an Octet-owned external-consumer fixture. Let Trellis consume a published immutable Octet revision in its own change.

Octet tests must not depend on a live Trellis checkout.

**Rationale:** A reusable provider needs local compatibility evidence and cannot rely on workspace-relative repository state.

### Decision: Defer code generation and migration

**Choice:** Exclude Eurydice and Scylla from this change. Record their later ownership and prerequisites in the roadmap.

**Rationale:** Rust-to-C export and C-to-Rust migration need different schemas, guarantees, failure classes, and evidence roles.

## Validation design

Positive evidence covers one non-fixture consumer profile, exact source selection, a named theorem, artifact validation, and Valence export.

Negative evidence covers:

- provider and compatibility identity drift
- copied or changed source
- missing, broad, duplicate, or ambiguous selected items
- partial or errorful LLBC and skipped borrow checking
- stale `translation.json` or generated Lean
- theorem, requirement, claim, or statement mismatch
- undeclared external models or trust inputs
- unsupported source, tool failure, and budget exhaustion classification
- stale receipt or Valence export identities
- weakened non-claims or role promotion

## Risks and trade-offs

- Consumer flexibility can weaken extraction policy. The provider profile remains closed and authoritative.
- External models can hide trust. Every model stays explicit and BLAKE3-bound.
- Charon IDs can change between runs. Durable identity stays bound to source bytes, spans, profiles, and generated artifacts.
- A downstream fixture can become a false compatibility promise. The contract remains versioned and revision-bound.
- The proof path adds maintenance cost. Expansion requires evidence from the first Trellis consumer.

## Claim boundary

A passing consumer receipt proves only the named theorem over the bound generated model, source, assumptions, tools, and artifacts.

It does not prove Aeneas, Charon, Lean, rustc, or translation soundness. It does not prove Verus model equivalence or downstream runtime behavior.
