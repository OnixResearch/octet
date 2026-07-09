## Tasks

- [ ] [serial] Define source-acquisition receipt DTOs and Nickel contracts. r[octet.dependency_trust.source_acquisition_receipts]
- [ ] [serial] Add positive fixtures for registry, public Git, approved private SSH, vendored snapshot, Nix flake, and local override sources. r[octet.dependency_trust.source_acquisition_receipts.positive]
- [ ] [serial] Add negative fixtures for unpinned Git, path-only release source, missing credential boundary, stale lock identity, and weakened non-claims. r[octet.dependency_trust.source_acquisition_receipts.negative]
- [ ] [serial] Implement pure source-acquisition validation and shell metadata extraction for Cargo/Nix inputs. r[octet.dependency_trust.source_acquisition_receipts.validation]
- [ ] [serial] Integrate source-acquisition receipts with artifact verification and dependency-trust summaries without merging evidence tiers. r[octet.dependency_trust.source_acquisition_receipts.evidence_separation]
- [ ] [serial] Document consumer guidance for stack-owned private SSH and local sibling overrides. r[octet.dependency_trust.source_acquisition_receipts.docs]
- [ ] [serial] Run focused tests, contract checks, artifact verification, and Cairn validation/gates for this change. r[octet.dependency_trust.source_acquisition_receipts.final_validation]
