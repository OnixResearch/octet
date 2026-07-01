## Contract

- [ ] [serial] Specify inline summary and content-addressed raw log retention rules. r[octet.evidence_log_storage.retention]
- [ ] [serial] Specify BLAKE3 manifest fields for bulky evidence artifacts. r[octet.evidence_log_storage.manifest]
- [ ] [serial] Specify fail-closed verification for missing or tampered externalized evidence. r[octet.evidence_log_storage.fail_closed]

## Implementation

- [ ] [serial] Add a manifest format and helper for content-addressed evidence logs. r[octet.evidence_log_storage.manifest]
- [ ] [serial] Convert selected bulky archive logs to summary-plus-manifest form without changing scoped claims. r[octet.evidence_log_storage.retention]
- [ ] [serial] Wire evidence log verification into the relevant release or lifecycle validation rail. r[octet.evidence_log_storage.fail_closed]

## Verification

- [ ] [serial] Add positive tests for a present artifact with matching BLAKE3 digest, byte count, command, and checked scope. r[octet.evidence_log_storage.manifest]
- [ ] [serial] Add negative tests for missing artifact, wrong digest, stale byte count, missing command, and missing checked scope. r[octet.evidence_log_storage.fail_closed]
- [ ] [serial] Run evidence verification, docs/link checks for transformed archive references, and Cairn validation/gates. r[octet.evidence_log_storage.retention]
