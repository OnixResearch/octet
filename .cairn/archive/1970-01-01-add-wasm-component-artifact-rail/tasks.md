# Tasks

## 1. wasm-tools cohort and typed profile policy

- [x] [serial] 1.1 r[octet.wasm_component.cohort] Pin a reviewed wasm-tools cohort covering at least `wasmparser`, `wasm-encoder`, `wat`, `wasmprinter`, `wasm-smith`, and `wasm-mutate` at one compatible version family.
- [x] [serial] 1.2 r[octet.wasm_component.profile] Define Nickel-authored artifact profiles for allowed proposals, artifact kinds, imports, exports, worlds, resources, memory-growth posture, metadata posture, severity mapping, non-claims, and verification role.
- [x] [serial] 1.3 r[octet.wasm_component.profile] Export the typed profile into a deterministic generated artifact consumed by the Rust checker and bind the export to a BLAKE3 identity.
- [x] [serial] 1.4 r[octet.wasm_component.cohort] Add freshness checks so stale profile exports or incomplete cohort pins fail validation.

## 2. Exact-byte Rust fact extraction

- [x] [serial] 2.1 r[octet.wasm_component.functional_core] Add a pure Rust core that parses exact final module or component bytes without execution.
- [x] [serial] 2.2 r[octet.wasm_component.validation] Extract normalized module/component kind, proposals, WIT imports and exports, worlds, producers metadata presence, custom-section inventory, and resource declarations.
- [x] [serial] 2.3 r[octet.wasm_component.validation] Detect malformed binaries, disallowed imports or worlds, resource-limit breaches, forbidden memory growth, forbidden start functions, and duplicate or contradictory metadata.
- [x] [serial] 2.4 r[octet.wasm_component.identity] Separate exact artifact identity from normalized interface identity and reject attempts to substitute one for the other.
- [x] [serial] 2.5 r[octet.wasm_component.functional_core] Keep file reads, CLI parsing, and artifact writes in a thin imperative shell around the pure checker core.

## 3. CLI and artifact bundle

- [x] [serial] 3.1 r[octet.wasm_component.findings] Add a standalone `cargo octet` artifact-validation command that consumes exact final bytes plus an explicit profile.
- [x] [serial] 3.2 r[octet.wasm_component.findings] Emit deterministic JSON facts and SARIF findings suitable for CI and editor integration.
- [x] [serial] 3.3 r[octet.wasm_component.findings] Emit profile identity, cohort identity, exact artifact identity, normalized interface identity, provenance, and explicit non-claims in the Octet evidence bundle.
- [x] [serial] 3.4 r[octet.wasm_component.identity] Extend `cargo octet artifact verify` to recompute all hashes and reject stale, missing, substituted, or tampered artifact inputs.
- [x] [serial] 3.5 r[octet.wasm_component.valence_export] Export the evidence through the standalone `valence-core` dependency with a bounded verification role.

## 4. Mantle exact-byte handoff

- [x] [serial] 4.1 r[octet.wasm_component.mantle_handoff] Define the handoff shape by which Mantle materializes exact final Wasm or component bytes for Octet validation.
- [x] [serial] 4.2 r[octet.wasm_component.mantle_handoff] Require the Octet receipt to bind the exact bytes supplied by Mantle rather than a source-tree or store-path proxy.
- [x] [serial] 4.3 r[octet.wasm_component.mantle_handoff] Preserve claim boundaries so Mantle's build or sandbox evidence and Octet's artifact-analysis evidence remain distinct.
- [x] [serial] 4.4 r[octet.wasm_component.mantle_handoff] Document a standalone path that does not require Mantle and an orchestrator path that can compose Mantle, Octet, and runtime evidence.

## 5. Fixtures and tests

- [x] [serial] 5.1 r[octet.wasm_component.fixtures] Add valid core-module and component fixtures with stable normalized interface facts.
- [x] [serial] 5.2 r[octet.wasm_component.fixtures] Add negative fixtures for malformed binaries, undeclared WASI imports, disallowed proposals, over-limit resources, forbidden growth, forbidden start behavior, and duplicate metadata.
- [x] [serial] 5.3 r[octet.wasm_component.fixtures] Add property tests for deterministic output, exact-vs-normalized identity separation, stable finding order, and fail-closed receipt verification.
- [x] [serial] 5.4 r[octet.wasm_component.fixtures] Add a pinned generated corpus lane using `wasm-smith`, with deterministic seeds, recorded transforms, expected issue classes, and shrink predicates.
- [x] [serial] 5.5 r[octet.wasm_component.fixtures] Add an optional deeper nightly mutation lane using `wasm-mutate`, with outcomes kept as validation or analysis evidence only.

## 6. Documentation, policy, and rollout

- [x] [serial] 6.1 r[octet.wasm_component.nonclaims] Document the artifact profile model, standalone CLI, Mantle handoff, identity split, known gaps, and evidence claim boundaries.
- [x] [serial] 6.2 r[octet.wasm_component.profile] Register the profile and receipt family in Octet's policy registry and lifecycle documentation.
- [x] [serial] 6.3 r[octet.wasm_component.final_validation] Add Nix checks for cohort pin completeness, profile freshness, fixture coverage, and exact-byte verification.
- [x] [serial] 6.4 r[octet.wasm_component.final_validation] Run focused Rust tests, Clippy, Nickel checks, artifact verification, and Cairn validation.
- [x] [serial] 6.5 r[octet.wasm_component.profile] Keep SpaceWasm-specific constraints in the dependent `add-spacewasm-mvp-artifact-profile` change rather than broadening this general rail.
