## Context

Octet currently has two intentionally distinct Rust-analysis surfaces: Dylint/HIR-based lints and a selected-source token closure with exact reviewed summaries. The Aeneas provider adds a third useful input: pinned Charon output with typed item and control-flow structure. This change adapts that output as a bounded recorded analysis rail, not as a compiler IR owned by Octet and not as a replacement for existing checks.

The prerequisite is a completed `add-aeneas-lean-proof-rail` extraction contract. This change must reuse its reviewed Charon package, effective-option validation, target binding, partial-output rejection, and upgrade canaries rather than creating a second pin or runner.

## Decisions

### 1. Keep the rail opt-in and separate

**Choice:** Expose Charon semantic analysis through the optional evidence-rail surface, with a provider ID such as `charon-semantic`, rather than changing `cargo octet check`, strict-profile admission, source-token closure, or `cargo octet prove`.

**Rationale:** Compiler-semantic snapshots have different support, cost, and trust boundaries from starter lints, source-token closure, and theorem checking.

### 2. Reuse one exact extraction authority

**Choice:** The rail consumes the reviewed Charon package/profile from the Aeneas provider. Admission requires exact effective serialized options, `has_errors = false`, `skip_borrowck = false`, target triple/pointer width/endianness, MIR/drop/start/selection/opacity/transformation settings, serialization format, and selected-item identities. Ambient tools, branch pins, or undocumented Cargo metadata are forbidden.

**Rationale:** Parallel Charon runners would drift and could attach analysis to partial or weakened extraction.

### 3. Keep parsing and analysis in a pure core

**Choice:** The shell measures and parses confined local artifacts, then passes typed in-memory snapshots to pure deterministic validators and analyses. The core performs no filesystem, process, environment, network, clock, or runtime Nickel access.

**Rationale:** Semantic decisions must be testable without standing up rustc, Charon, Cargo, or a workspace.

### 4. Bound the initial semantic fact vocabulary

**Choice:** The first profile supports exact item selection, item kind and source span, resolved local call edges when present, declaration dependencies, trait-resolution rows, structured/unstructured control-flow shape, recursion/cycles, opacity/unsupported markers, and direct effect candidates derived only from a closed reviewed mapping over explicit IR constructs. Missing or unsupported facts become `unknown` or blocked; they are never inferred from names.

**Rationale:** A small closed vocabulary is reviewable. General Rust semantic closure, alias analysis, unsafe-dataflow proof, and runtime reachability are out of scope.

### 5. Compare evidence without merging authority

**Choice:** A pure comparison core may align Charon item/span facts with Octet function-address and source-token closure artifacts and report `agree`, `disagree`, `charon-only`, `source-token-only`, or `unknown`. The report preserves both parent identities and never rewrites either parent receipt or promotes agreement to semantic equivalence.

**Rationale:** Differential evidence is useful precisely when disagreement and uncertainty remain visible.

### 6. Emit role-separated BLAKE3 evidence

**Choice:** Receipts bind the Charon package/profile, effective options, target, exact source and selected items, raw snapshot, normalized semantic projection, analysis profile, comparison parents, diagnostics, limits, and non-claims with role-separated BLAKE3 identities. Display paths and tool-local declaration numbers are review metadata only.

**Rationale:** Reproducible identity is required, but path or local-number stability must not be confused with semantic stability.

### 7. Keep upstream references non-authoritative

**Choice:** Record exact reviewed `mechanized-llbc` and `charon-rudra` revisions, licenses, and sentinels as reference metadata. Charon-Rudra analysis patterns may inform local fixtures, but Octet does not depend on its old runtime/toolchain. Mechanized LLBC remains a semantic corpus, not a conformance certificate for current Charon.

**Rationale:** Both repositories are valuable review inputs but cannot transfer current implementation or proof claims.

### 8. Keep Valence and Kamacite at their existing boundaries

**Choice:** Valence may validate exported artifact identities, roles, links, and non-claims. Kamacite may carry LLBC or semantic projections as target/provenance artifacts. Neither becomes a Charon/LLBC compiler IR or semantic-analysis authority.

**Rationale:** Octet owns Rust analysis; Valence owns evidence semantics; Kamacite owns interchange identity.

## Risks / Trade-offs

- Charon support gaps may block common crates. Exact-item blocked receipts and canaries are accepted outcomes.
- Resolved compiler facts may still be wrong if rustc or Charon is wrong. Receipts bind observations and do not claim compiler correctness.
- Comparing source-token and Charon facts can encourage false equivalence. The closed comparison statuses and non-claims prevent silent merging.
- Charon schema and option changes can be frequent. The rail inherits the provider's fixture-driven upgrade matrix and adds semantic-projection canaries.
