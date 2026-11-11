# Tasks: Extend the Aeneas consumer proof budget

- [x] [serial] Raise the consumer, provider authority, rail, and live-test timeouts and regenerate the profile. r[octet.aeneas_consumer_contract.live_budget_extension]
- [x] [serial] Update the profile identity, expected receipt, and fixture timeout fields. r[octet.aeneas_consumer_contract.live_budget_extension]
- [x] [serial] Run focused Aeneas tests and the Aeneas consumer job. r[octet.aeneas_consumer_contract.live_budget_extension]
- [x] [serial] Refresh the self-check baseline if needed and run the full flake check. r[octet.aeneas_consumer_contract.live_budget_extension]

## Verification Coverage

- Aeneas job exit 0; full flake check all checks passed. Evidence: `evidence/aeneas-job.json`.

- `Scenario: Cold Lean closure completes within the extended budget` -> `mantle-aeneas-consumer-job`
