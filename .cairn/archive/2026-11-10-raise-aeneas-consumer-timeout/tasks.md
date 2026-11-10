# Tasks: Raise the Aeneas consumer proof budget

- [x] [serial] Raise the external-consumer timeout and regenerate the profile. r[octet.aeneas_consumer_contract.live_budget]
- [x] [serial] Raise the provider authority maximum and the selected Aeneas rail timeout. r[octet.aeneas_consumer_contract.live_budget]
- [x] [serial] Update the profile identity, expected receipt, and Aeneas fixture artifacts. r[octet.aeneas_consumer_contract.live_budget]
- [x] [serial] Run the Aeneas consumer job check from a cold build. r[octet.aeneas_consumer_contract.live_budget]
  - exit 0; evidence: `evidence/aeneas-consumer-job.json`.
- [x] [serial] Run focused tests, refresh the self-check baseline, and run the full flake check. r[octet.aeneas_consumer_contract.live_budget]
  - focused tests: 41 `octet-standards`, 864 `cargo-octet`, 4 live compatibility; evidence: `evidence/focused-tests.json`.
  - self-check baseline: 3,107 identities / 4,262 warnings, hash `b3:8aa117…`; evidence: `evidence/self-check-refresh.json`.
  - full flake check: exit 0, all checks passed; evidence: `evidence/full-flake-check.json`.

## Verification Coverage

- `Scenario: Cold Lean closure completes within the consumer budget` -> `mantle-aeneas-consumer-job` check
