## Contract

- [ ] [serial] Specify Rust-owned generated Nickel contracts for emitted finding and provenance DTOs. r[octet.evidence_contracts.generated]
- [ ] [serial] Specify freshness registration for generated evidence contracts. r[octet.evidence_contracts.registry]
- [ ] [serial] Specify fail-closed contract fixtures for malformed evidence records. r[octet.evidence_contracts.fail_closed]

## Implementation

- [ ] [serial] Add a deterministic Rust-owned schema source and generator for `Finding` v2 and `ProvenanceRecord`. r[octet.evidence_contracts.generated]
- [ ] [serial] Commit generated Nickel contracts and wire them into policy validation freshness checks. r[octet.evidence_contracts.registry]
- [ ] [serial] Update contributor and policy docs to state the Rust-derived ownership rule for emitted evidence DTOs. r[octet.evidence_contracts.generated]

## Verification

- [ ] [serial] Add positive fixtures for valid finding and provenance records. r[octet.evidence_contracts.generated]
- [ ] [serial] Add negative fixtures for missing required fields, invalid evidence tier, unsupported schema version, and wrong field types. r[octet.evidence_contracts.fail_closed]
- [ ] [serial] Run contract freshness checks, policy validation, targeted cargo-octet tests, and Cairn validation/gates. r[octet.evidence_contracts.registry]
