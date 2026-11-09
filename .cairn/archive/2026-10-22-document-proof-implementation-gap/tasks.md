## Phase 1: Documentation

- [x] [serial] Add `docs/proof-implementation-gap.md` with the four specification assumptions named verbatim and the sentence that a verified model is not a verified implementation r[octet.proof_implementation_gap.assumptions]
- [x] [serial] Document the three layers interleaving, fault handling, and unverified extensions in the failure-class guide, each with the specification assumptions at risk and the system-level testing that closes it r[octet.proof_implementation_gap.layers]
- [x] [serial] State the honest claim boundary in the failure-class guide, including that no existing proof is claimed unsound and that the closing system-level testing does not exist in this repository r[octet.proof_implementation_gap.claim_boundary]
- [x] [serial] Record the reporting shape for a defect in the class, naming the violated assumption, the layer, the observed and modeled behavior, and the reproducing test or open obligation r[octet.proof_implementation_gap.reporting]
- [x] [parallel] Link the failure class from `README.md`, `docs/trust-boundary-evidence.md`, and `docs/contributor-guide.md` r[octet.proof_implementation_gap.reporting]
- [x] [serial] Extend the law-tested deterministic cores guide with the boundary of the law doctrine and a link to the failure class r[octet.proof_implementation_gap.property_testing_boundary]
- [x] [parallel] Add the proof-to-implementation boundary section to the standards verification chapter without changing the existing policy-anchored sections r[octet.proof_implementation_gap.assumptions]

## Phase 2: Specification

- [x] [serial] Add the `proof-implementation-gap` capability delta with the assumption, layer, claim-boundary, property-testing-boundary, and reporting requirements r[octet.proof_implementation_gap.assumptions]
- [x] [serial] Modify the accepted `law-tested-deterministic-cores` requirement so law and property tests are never presented as implementation evidence r[octet.law_tested_deterministic_cores.fixture_complement]

## Phase 3: Verification

- [x] [serial] Verify that the failure-class guide names all four specification assumptions, all three layers, and the model-versus-implementation sentence r[octet.proof_implementation_gap.assumptions]
- [x] [parallel] Verify that the failure-class guide is reachable from the README, the trust-boundary evidence index, the contributor guide, and the law-tested deterministic cores guide r[octet.proof_implementation_gap.reporting]
- [x] [serial] Verify that the failure-class guide states that no existing proof is claimed unsound and that the closing system-level testing does not exist in this repository r[octet.proof_implementation_gap.claim_boundary]
- [x] [serial] Run the Cairn proposal, design, and tasks gates and the strict lifecycle validation r[octet.proof_implementation_gap.property_testing_boundary]
- [x] [serial] Sync the delta specs into the accepted capabilities and archive the change r[octet.proof_implementation_gap.reporting]
