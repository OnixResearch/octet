## Phase 1: Provider contract and compatibility

- [x] [serial] Define the common formal-proof envelope and closed `aeneas-lean` provider payload while preserving existing Verus v1 receipt bytes through an explicit adapter. r[octet.aeneas_lean_proof_rail.provider_contract] r[octet.aeneas_lean_proof_rail.verus_compatibility]
- [x] [serial] Add the typed Nickel provider profile and generated runtime export for exact Aeneas, Charon pin, OCaml, Lean, Aeneas Lean library, supported source surface, argv, limits, environment, effective extraction policy, assumptions, semantic-reference metadata, and non-claims. r[octet.aeneas_lean_proof_rail.provider_contract] r[octet.aeneas_lean_proof_rail.semantic_reference]
- [x] [parallel] Add compatibility goldens proving existing Verus package locks, receipts, proof cards, and artifact verification remain unchanged. r[octet.aeneas_lean_proof_rail.verus_compatibility]
- [x] [parallel] Pin and validate one exact `mechanized-llbc` revision, license, and sentinel as recorded-only semantic-foundation metadata without importing proof authority. r[octet.aeneas_lean_proof_rail.semantic_reference]

## Phase 2: Pure stage-chain validation

- [ ] [serial] Implement pure validation for ordered Rust source, effective Charon extraction contract, LLBC, Aeneas `translation.json`, generated Lean, theorem source, external models, adequacy decision, trust closure, and Lean-check stage identities. r[octet.aeneas_lean_proof_rail.stage_chain]
- [ ] [serial] Reject `has_errors`, `skip_borrowck`, target metadata drift, unreviewed MIR/drop/start/include/exclude/opaque/transform settings, and serialization-profile drift before property promotion. r[octet.aeneas_lean_proof_rail.charon_contract]
- [ ] [serial] Validate `translation.json` declaration links and generated-file inventory while treating `def_id` values as within-run join keys rather than durable identities. r[octet.aeneas_lean_proof_rail.translation_manifest]
- [ ] [serial] Require a bound named theorem, approved theorem-statement BLAKE3, requirement ID, claim class, complete trust closure, adequacy decision, and passing Lean checker result before `formal_proof` promotion; keep translation-only outputs recorded-only. r[octet.aeneas_lean_proof_rail.theorem_gate] r[octet.aeneas_lean_proof_rail.theorem_adequacy]
- [ ] [serial] Classify unsafe, concurrent, unsupported loop-control, unresolved external, and translation-failure inputs as blocked or failed without proof promotion. r[octet.aeneas_lean_proof_rail.unsupported_surface]
- [ ] [parallel] Add negative stage-chain cases for stale source, partial/errorful LLBC, borrow-check bypass, target/profile drift, swapped LLBC, stale or mismatched `translation.json`, stale generated Lean, unstable `def_id` reuse, wrong tool pin, missing theorem, vacuous `True` theorem, `sorry`/`admit`, undeclared model or axiom, weakened non-claims, and role promotion. r[octet.aeneas_lean_proof_rail.stage_chain] r[octet.aeneas_lean_proof_rail.charon_contract] r[octet.aeneas_lean_proof_rail.translation_manifest] r[octet.aeneas_lean_proof_rail.theorem_gate] r[octet.aeneas_lean_proof_rail.theorem_adequacy]

## Phase 3: Imperative execution shell and package

- [ ] [serial] Package the exact provider closure and assert that the configured Charon revision matches Aeneas's reviewed pin before execution. r[octet.aeneas_lean_proof_rail.safe_execution]
- [ ] [serial] Add argv-only Charon, Aeneas `-emit-json`, and Lean execution with isolated work roots, named time/resource/log limits, an environment allowlist, and no network access. r[octet.aeneas_lean_proof_rail.safe_execution]
- [ ] [serial] Add a fixture-driven compatibility gate for every Aeneas, Charon, rustc, Lean, library, schema, or profile upgrade, covering positive, false, vacuous, partial-output, borrow-check bypass, opacity/model, target, freshness, and unsupported-feature canaries. r[octet.aeneas_lean_proof_rail.pin_upgrade_matrix]
- [ ] [parallel] Add process-failure fixtures for spawn failure, timeout, bounded-log overflow, nonzero translation, nonzero theorem check, and accidental selection of the false theorem. r[octet.aeneas_lean_proof_rail.safe_execution] r[octet.aeneas_lean_proof_rail.final_validation]

## Phase 4: Proof artifacts and stack handoff

- [ ] [serial] Extend proof cards and artifact verification to display and revalidate the effective extraction contract, `translation.json`, generated/maintained source boundary, theorem statement hash, requirement ID, claim class, adequacy decision, trust closure, assumptions, tool identities, and scoped claim. r[octet.aeneas_lean_proof_rail.artifact_evidence]
- [ ] [serial] Export a frozen Valence adapter fixture only after the Octet receipt and artifact directory verify. r[octet.aeneas_lean_proof_rail.valence_handoff]
- [ ] [parallel] Add tamper tests for every exported stage digest, role, status, theorem ID, and receipt link. r[octet.aeneas_lean_proof_rail.artifact_evidence] r[octet.aeneas_lean_proof_rail.valence_handoff]

## Phase 5: Documentation and validation

- [ ] [serial] Document provider selection, supported subset, assumption review, theorem requirements, rollout order, and explicit non-claims without changing starter `cargo octet check` behavior. r[octet.aeneas_lean_proof_rail.docs]
- [ ] [serial] Run the positive provider fixture, false and vacuous theorem fixtures, extraction-contract and translation-manifest negatives, every other declared negative fixture, the complete pin-upgrade compatibility matrix, existing Verus production positive/negative rails, policy freshness, artifact verification, focused tests, and Cairn validation/gates. r[octet.aeneas_lean_proof_rail.final_validation] r[octet.aeneas_lean_proof_rail.pin_upgrade_matrix]
