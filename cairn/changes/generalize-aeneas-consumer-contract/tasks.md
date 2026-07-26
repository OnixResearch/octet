# Tasks: Generalize the Aeneas consumer contract

## Phase 1: Typed consumer policy

- [ ] [serial] I1 Add the closed Nickel consumer-profile contract and deterministic generated fixture profile. r[octet.aeneas_consumer_contract.profile]
- [ ] [serial] I2 Extend proof-rail metadata and pure validation with consumer, selected-item, theorem, requirement, claim, model, and trust identities. r[octet.aeneas_consumer_contract.profile] r[octet.aeneas_consumer_contract.theorem]
- [ ] [parallel] I3 Add invalid profiles for provider drift, unknown fields, path escapes, empty non-claims, duplicate items, and incomplete theorem mappings. r[octet.aeneas_consumer_contract.profile] r[octet.aeneas_consumer_contract.negative_evidence]

## Phase 2: Exact-source execution

- [ ] [serial] I4 Adapt Aeneas execution to consume the selected consumer rail through existing policy artifacts and explicit source roots. r[octet.aeneas_consumer_contract.execution]
- [ ] [serial] I5 Bind selected source bytes and spans through Charon, LLBC, `translation.json`, generated Lean, and maintained theorem source. r[octet.aeneas_consumer_contract.source_selection]
- [ ] [serial] I6 Implement deterministic `supported`, `blocked`, and `failed` classification in the pure core. r[octet.aeneas_consumer_contract.outcomes]
- [ ] [parallel] I7 Add process, budget, unsupported-source, ambiguous-selection, shadow-source, stale-output, and trust-drift negative cases. r[octet.aeneas_consumer_contract.outcomes] r[octet.aeneas_consumer_contract.negative_evidence]

## Phase 3: Evidence handoff

- [ ] [serial] I8 Emit consumer-bound receipts and proof cards with exact provider, profile, source, theorem, requirement, trust, and artifact identities. r[octet.aeneas_consumer_contract.artifacts]
- [ ] [serial] I9 Export a verified consumer receipt through the existing Valence Aeneas chain adapter without role promotion. r[octet.aeneas_consumer_contract.valence_handoff]
- [ ] [parallel] I10 Add receipt, artifact, role, and Valence-export tamper fixtures. r[octet.aeneas_consumer_contract.artifacts] r[octet.aeneas_consumer_contract.valence_handoff] r[octet.aeneas_consumer_contract.negative_evidence]

## Phase 4: Compatibility and documentation

- [ ] [serial] I11 Add an Octet-owned external-consumer fixture that does not depend on a live sibling checkout. r[octet.aeneas_consumer_contract.compatibility]
- [ ] [parallel] V1 Run the positive consumer fixture plus false and vacuous theorem negatives. r[octet.aeneas_consumer_contract.negative_evidence]
- [ ] [parallel] V2 Run consumer-profile, source-selection, outcome, artifact, and Valence tamper matrices. r[octet.aeneas_consumer_contract.final_validation]
- [ ] [parallel] V3 Run the pinned provider compatibility matrix, existing Aeneas fixture rail, existing Verus rails, and starter-path independence tests. r[octet.aeneas_consumer_contract.compatibility]
- [ ] [serial] V4 Document downstream adoption, Trellis ownership, revision pinning, and deferred Eurydice and Scylla boundaries. r[octet.aeneas_consumer_contract.boundaries]
- [ ] [serial] V5 Run focused Cargo tests, artifact validation, policy generation checks, Cairn validation, and all change gates. r[octet.aeneas_consumer_contract.final_validation]

## Verification coverage

- `Scenario: Complete consumer profile is accepted` -> I1, I2, I11, V1
- `Scenario: Consumer profile weakens provider authority` -> I3, V2
- `Scenario: Exact selected item reaches the generated model` -> I4, I5, I11, V1
- `Scenario: Source or item selection is ambiguous` -> I7, V2
- `Scenario: Supported consumer proof completes` -> I4, I6, I8, V1
- `Scenario: Unsupported source is blocked` -> I6, I7, V2
- `Scenario: Tool or policy error fails` -> I6, I7, V2
- `Scenario: Maintained theorem is admitted` -> I2, I5, I8, V1
- `Scenario: Theorem or trust mapping drifts` -> I3, I7, V2
- `Scenario: Consumer artifact bundle is valid` -> I8, V3
- `Scenario: Artifact or role is stale` -> I10, V2
- `Scenario: Verified consumer receipt exports` -> I9, V3
- `Scenario: Unverified consumer receipt cannot export property evidence` -> I10, V2
- `Scenario: Existing proof rails retain behavior` -> I11, V3
- `Scenario: Roadmap preserves ownership and non-claims` -> V4, V5
- `Scenario: Final consumer matrix is complete` -> V1, V2, V3, V5
