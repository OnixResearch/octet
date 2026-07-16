## Context

`ProofRail` already carries a backend label and shell-free command argv, but the durable proof package lock and receipt types are named and shaped around Verus. Aeneas cannot be admitted safely by changing only `backend = "aeneas"`: its trusted and reviewable path contains multiple generated artifacts and two distinct semantic tools before the final proof assistant runs.

The initial provider is deliberately `aeneas-lean`, not a promise to support every Aeneas backend. Lean is selected because Aeneas documents it as a mature backend with a maintained library and tutorial. HOL4, F*, and Coq remain future providers requiring their own receipts and fixtures. The reviewed compatibility baseline uses Aeneas's exact Charon pin `cb50ff16b9f1066b8a97dc06da704de2da2fa41c`; later pins are admitted only through the compatibility matrix defined below.

## Decisions

### 1. Add a common envelope without erasing provider differences

**Choice:** Introduce a versioned common formal-proof envelope for policy ID, rail ID, provider ID, checked scope, status, source bindings, theorem/property ID, package-lock identity, artifact identities, trust boundaries, and non-claims. Provider-specific payloads remain closed typed records. The existing Verus receipt is adapted into the common envelope without changing accepted Verus v1 bytes or authority.

**Rationale:** Shared orchestration and proof-card code should not duplicate common facts, but flattening Aeneas into Verus fields would hide its translation stages.

### 2. Model Aeneas as an ordered, hash-linked stage chain

**Choice:** The `aeneas-lean` payload binds these stages in order:

1. exact checked Rust source artifacts and selected item scope;
2. rustc and Charon identity, Aeneas preset argv, effective serialized Charon options, target metadata, extraction status, and emitted LLBC BLAKE3;
3. Aeneas identity, backend/config argv, and emitted `translation.json` BLAKE3;
4. generated Lean definitions, generated-file inventory, imported model inventory, and their BLAKE3 identities;
5. hand-maintained Lean theorem source, approved theorem-statement BLAKE3, requirement ID, claim class, adequacy decision, and trust closure;
6. Lean toolchain, Aeneas Lean library, package lock, checker argv, and final result.

Every stage records input and output identities. A downstream stage is valid only when its declared input matches the previous stage's output. Display paths are review metadata; BLAKE3 identities and typed roles are canonical.

**Rationale:** A final `lake` exit code cannot detect a stale LLBC file or generated Lean file that was produced from different Rust bytes.

### 3. Bind the effective Charon extraction contract

**Choice:** The provider records and validates the effective serialized Charon options after command-line, preset, and Cargo-metadata composition. Property promotion requires `has_errors = false`, `skip_borrowck = false`, exact target triple/pointer width/endianness, reviewed MIR start stage and drop policy, exact `start_from`, include/exclude/opaque rules, transformation settings, and serialization format. Missing, unknown, or weakened extraction settings fail before theorem execution.

**Rationale:** A valid Lean theorem must not be attached to partial LLBC, hidden crate metadata, a different target model, or extraction that bypassed Rust borrow checking.

### 4. Treat `translation.json` as a proof-relevant stage

**Choice:** The Aeneas `translation.json` manifest is a distinct measured artifact. Validation joins its LLBC declaration IDs to selected LLBC items and generated Lean declarations, and preserves Rust names/spans, Lean names/files, `is_opaque`, `can_fail`, `can_diverge`, recursion, and loop-parent metadata. Declaration IDs are valid only as within-run join keys and never become durable Valence or Octet stable IDs.

**Rationale:** The manifest is the only explicit declaration-level bridge from LLBC to generated Lean. Durable identity must survive tool-local numbering changes without losing exact source/model linkage.

### 5. Gate property promotion on theorem adequacy and trust closure

**Choice:** The provider requires a reviewed requirement ID, approved theorem-statement BLAKE3, explicit claim class, generated-versus-maintained source disposition, declared assumptions/models/opaque definitions, and a deterministic adequacy decision before theorem execution can yield `formal_proof`. Selected theorem closure must reject `sorry`, `admit`, undeclared axioms, missing external models, and vacuous statements that do not satisfy the approved claim class. A `jxl-proofs`-style `True` postcondition is a required negative adequacy fixture.

**Rationale:** Lean accepting a theorem establishes only that the encoded statement checks. Octet must separately bind and review whether that statement is the approved property and whether its trust boundary is complete.

### 6. Generated translation is not theorem discharge

**Choice:** Successful Charon extraction, Aeneas translation, or compilation of generated Lean definitions remains `recorded_only`. Octet emits `formal_proof` only when the configured named theorem is present, the theorem source is bound, Lean checks it successfully, and the complete stage chain validates.

**Rationale:** Translating a function into Lean does not prove the property the consumer cares about.

### 7. Assumptions and external models are first-class inputs

**Choice:** The provider payload lists every hand-written external model, axiom/assumption declaration, admitted theorem, and unsupported item disposition with exact identity and review role. Undeclared or stale model inputs fail closed. Empty assumption inventories are explicit rather than inferred.

**Rationale:** Aeneas projects external Rust definitions through hand-written backend models; omitting those models from identity would make proof receipts misleading.

### 8. Unsupported Rust remains an explicit preflight result

**Choice:** The provider profile admits only the reviewed safe sequential subset. Unsafe code, concurrency, unsupported nested-loop control transfer, unresolved externals, translation failure, or unsupported features produce typed blocked/failed results and never `formal_proof` provenance.

**Rationale:** Unsupported source is not a failed theorem and must not be confused with a proved absence of bugs.

### 9. Shell execution is bounded and deterministic

**Choice:** The CLI shell resolves pinned executables from the provider package, runs explicit argv in an isolated work root, applies named time/log/resource limits, and captures bounded output artifacts. The pure core receives typed stage observations and validates package locks, links, statuses, scopes, and claims. No shell command strings, ambient PATH resolution, network access, or runtime Nickel evaluation are introduced.

**Rationale:** This preserves Octet's functional-core/imperative-shell boundary and existing proof execution posture.

### 10. Start with an Octet-owned plumbing theorem

**Choice:** The first fixture is a tiny dependency-free safe-Rust function with a directly stated Lean theorem, an intentionally false sibling theorem, and a vacuous `True` theorem outside the positive selection scope. Promotion beyond fixture mode requires a production package identity, both negative selections failing for their declared reasons, stale-stage and extraction-contract tamper rejection, and artifact verification.

**Rationale:** Consumer code should not be the first place the provider's failure semantics are tested.

### 11. Valence consumes a frozen verified adapter fixture

**Choice:** Octet exports an Aeneas translation-chain adapter input only after its own receipt and artifact directory verify. Valence receives exact fixture bytes and does not rerun the tools. The export preserves each intermediate stage as recorded-only/boundary evidence and the final Octet-owned theorem receipt as the only property candidate.

**Rationale:** Octet owns execution and proof receipt truth; Valence owns canonical cross-stack identity and role linkage.

### 12. Upgrades are fixture-gated and semantic references remain non-authoritative

**Choice:** Aeneas, Charon, rustc, Lean, library, schema, or profile upgrades rerun the positive theorem, false-theorem rejection, vacuity rejection, partial-output rejection, borrow-check bypass rejection, opacity/external-model cases, target-width/endianness cases, full source-to-manifest-to-Lean freshness, and unsupported-feature canaries. The provider profile may record an exact `mechanized-llbc` revision, license, and source sentinel as `recorded_only` semantic-foundation metadata; it never treats that corpus as proof of the pinned binaries.

**Rationale:** Charon is alpha and its documented support surface can lag implementation changes. Compatibility is established by exact fixtures, not branch availability, issue status, or semantic-reference prose.

## Rollout Order

1. Octet fixture provider and fail-closed receipt validation.
2. Valence translation-chain adapter over a frozen Octet fixture.
3. Trellis exact-source cross-verifier feasibility pilot.
4. UCAN `TimeBounds::new` downstream pilot.

Mantle can already carry the resulting Valence/Octet sidecars through generic external evidence. Cairn can already consume generic evidence-chain readiness inputs. Neither needs an Aeneas-specific provider in this slice.

## Risks / Trade-offs

- Generalizing receipt infrastructure can accidentally weaken Verus validation. Compatibility goldens and Verus positive/negative rails are mandatory.
- Generated Lean output can be deterministic while still modeling the wrong semantics. Receipts therefore bind identity and theorem execution but explicitly do not claim translation or verifier soundness.
- External model inventories can grow quickly. The first fixture must stay dependency-free and bounded.
- Aeneas, Charon, rustc, and Lean pins may move together. Upgrades require a new compatibility receipt and complete fixture matrix rather than package availability, documentation, or issue status alone.
- Charon declaration IDs may change without semantic source changes. They remain within-run links and never substitute for BLAKE3-bound durable identities.
- Theorem adequacy checks can detect declared vacuity and trust-boundary drift but cannot prove that a human-selected requirement is the right product requirement.
- The safe subset may reject intended consumers. A blocked feasibility result with an exact unsupported construct is an acceptable outcome; shadow implementations or weakened claims are not.
