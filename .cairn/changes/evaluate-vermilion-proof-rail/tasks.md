# Tasks: Evaluate Vermilion alongside Verus and Aeneas proof rails

## Provider profile

- [x] [serial] Add closed Nickel contracts for provider pins, compatibility canaries, the pilot semantic contract, and the negative matrix. r[octet.vermilion_proof_rail.provider_profile]
- [x] [serial] Add the pinned Vermilion profile and compatibility matrix, then export deterministic JSON. r[octet.vermilion_proof_rail.provider_profile]
- [x] [serial] Register the Nickel-authored provider and pilot-contract families in the policy registry. r[octet.vermilion_proof_rail.provider_profile]
- [x] [serial] Build the pinned Vermilion/Verus/Lean/cvc5 closure inside an isolated execution environment. Lean 4.33.0, the mathlib cache, Vermilion Lean libraries, the Rust workspace, and the pinned Verus fork built; see `standards/proof-rails/vermilion-lean/evidence/2026-11-09-live-canary-simple.md`. r[octet.vermilion_proof_rail.provider_profile]
- [x] [parallel] Run the live compatibility canaries and save a fresh provider receipt. The upstream `m1-pipeline/simple.rs` canary passed 3/3 Lean obligations with 0 sorry and a kernel-checked proof twin; the u64 pilot receipt remains open. See the live-canary evidence. r[octet.vermilion_proof_rail.provider_profile]

## Semantic contract and subject

- [x] [serial] Add the `increment_checked` pilot subject with explicit checked overflow and the canonical `value < u64::MAX` statement bytes. r[octet.vermilion_proof_rail.semantic_contract]
- [x] [serial] Declare Verus, Vermilion, and Aeneas route obligations and open refinement bridges. r[octet.vermilion_proof_rail.semantic_contract]
- [x] [serial] Add a comparison test proving the existing Verus `nat` lemma is incomparable with the u64 contract. r[octet.vermilion_proof_rail.semantic_contract]
- [ ] [serial] Check a Verus refinement theorem for the u64 checked subject. Blocked: not attempted; the existing rail proves only a `nat` lemma. r[octet.vermilion_proof_rail.semantic_contract]
- [ ] [serial] Check Aeneas and Vermilion refinement bridges to the executable function. Blocked: no live provider obligation or theorem exists yet. r[octet.vermilion_proof_rail.semantic_contract]

## Receipt and obligations

- [x] [serial] Add typed provider, contract, source, toolchain, command, artifact, obligation, bridge, checker, inventory, outcome, and comparison DTOs. r[octet.vermilion_proof_rail.receipt]
- [x] [serial] Add fail-closed receipt validation for source, spec, overflow, width, provider, artifact, twin, obligation, axiom, hole, checker, dependency, command, tamper, non-claim, and authority classes. r[octet.vermilion_proof_rail.obligations]
- [x] [serial] Add the positive synthetic receipt fixture with sealed identity and `recorded_only` disposition. r[octet.vermilion_proof_rail.receipt]
- [x] [parallel] Add the twenty-case negative matrix and exercise every declared mutation. r[octet.vermilion_proof_rail.negative_controls]
- [ ] [serial] Emit a measured receipt from a real Vermilion run. Blocked: the provider closure is unavailable and no obligations were generated. r[octet.vermilion_proof_rail.receipt]

## Comparison and documentation

- [x] [serial] Add the closed comparison vocabulary and explicit Aeneas/Vermilion outcome mappings. r[octet.vermilion_proof_rail.comparison]
- [x] [parallel] Add comparison tests for agreement, semantic mismatch, incomparable claims, proof failure, blocked, timeout, tool error, and open bridges. r[octet.vermilion_proof_rail.comparison]
- [x] [serial] Document the claim/trust matrix, open obligations, blockers, and no-go recommendation. r[octet.vermilion_proof_rail.trust_boundary]
- [ ] [serial] Wire Vermilion receipts into `cargo octet prove`, proof cards, artifact verification, and Valence export. Deferred: requires a measured provider run and a separate reviewed change. r[octet.vermilion_proof_rail.trust_boundary]

## Verification and closeout

- [x] [serial] Run the focused `octet-standards` Vermilion tests. r[octet.vermilion_proof_rail.negative_controls]
- [x] [serial] Run `cargo fmt --check`. r[octet.vermilion_proof_rail.trust_boundary]
- [x] [serial] Run the workspace `cargo check --workspace --all-targets`. r[octet.vermilion_proof_rail.trust_boundary]
- [x] [serial] Run Clippy for `octet` and `cargo-octet` with warnings denied. r[octet.vermilion_proof_rail.trust_boundary]
- [x] [serial] Extend the proof-rail manifest check with Vermilion profile, contract, and matrix freshness assertions, and exercise the exported-body assertions locally. r[octet.vermilion_proof_rail.provider_profile]
- [x] [serial] Run `cairn validate`, proposal, design, and tasks gates. r[octet.vermilion_proof_rail.trust_boundary]
- [x] [serial] Commit the reviewed change. r[octet.vermilion_proof_rail.trust_boundary]

## Verification Coverage

- `Scenario: Complete profile is accepted` -> provider profile tests
- `Scenario: Provider pin drifts` -> profile mutation tests
- `Scenario: Routes align with the common contract` -> comparison agreement test
- `Scenario: A nat claim substitutes for u64` -> nat/u64 incomparable test
- `Scenario: Complete receipt is accepted` -> complete contract fixture test
- `Scenario: Stale generated output or proof twin is supplied` -> negative matrix
- `Scenario: Omitted obligation or unexpected axiom is present` -> negative matrix
- `Scenario: Checker cannot report a complete result` -> negative matrix and absent-toolchain test
- `Scenario: All routes check the same property` -> comparison agreement test
- `Scenario: Providers disagree or cannot execute` -> comparison category tests
- `Scenario: Negative matrix runs` -> negative matrix test
- `Scenario: External closure is absent` -> absent-toolchain test
- `Scenario: Synthetic fixture requests property evidence` -> negative matrix
