## Phase 1: Policy and contracts

- [x] [serial] Define the typed Nickel SARIF producer profile with supported version, identities, location policy, severity mappings, limits, and non-claims r[octet.external_lint.sarif_intake_profile]
- [x] [serial] Define Rust-owned SARIF intake receipt and normalized finding DTOs with generated Nickel contracts r[octet.external_lint.sarif_intake_identity]
- [x] [serial] Register the SARIF profile, raw report, normalized projection, receipt, and provenance families with explicit ownership r[octet.external_lint.sarif_intake_identity]

## Phase 2: Core and shell

- [x] [serial] Implement a pure bounded parser and validator for the supported SARIF subset r[octet.external_lint.sarif_intake]
- [x] [serial] Implement deterministic normalized finding IDs while preserving producer rule and severity identity r[octet.external_lint.sarif_intake_identity]
- [x] [serial] Add the capability-rooted `sarif-import` shell without hosted execution or network authority r[octet.external_lint.sarif_intake_boundary]
- [x] [serial] Extend artifact validation and Valence export with raw-report and normalized-projection parent linkage r[octet.external_lint.sarif_intake_identity]

## Phase 3: Verification

- [x] [parallel] Add positive clean and finding fixtures for distinct declared producer profiles r[octet.external_lint.sarif_intake_verification]
- [x] [parallel] Add negative fixtures for malformed data, unsupported versions, missing identities, unsafe locations, bad indexes, duplicates, stale hashes, and limits r[octet.external_lint.sarif_intake_verification]
- [x] [parallel] Add a controlled-deployment fixture that proves no hosted scanner or network service is required r[octet.external_lint.sarif_intake_boundary]
- [x] [parallel] Add tier and severity fixtures that reject checker relabeling and formal-proof promotion r[octet.external_lint.sarif_intake_boundary]

## Phase 4: Documentation and closeout

- [x] [parallel] Document the supported SARIF subset, producer profiles, local-only import, field preservation, limits, and non-claims r[octet.external_lint.sarif_intake_boundary]
- [x] [serial] Run focused tests, Nickel freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.external_lint.sarif_intake_verification]
- [x] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.external_lint.sarif_intake_profile]
