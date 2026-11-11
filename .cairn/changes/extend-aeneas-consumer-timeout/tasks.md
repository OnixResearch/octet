# Tasks: Extend the Aeneas consumer proof budget

- [ ] [serial] Raise the consumer, provider authority, rail, and live-test timeouts and regenerate the profile. r[octet.aeneas_consumer_contract.live_budget_extension]
- [ ] [serial] Update the profile identity, expected receipt, and fixture timeout fields. r[octet.aeneas_consumer_contract.live_budget_extension]
- [ ] [serial] Run focused Aeneas tests and the Aeneas consumer job. r[octet.aeneas_consumer_contract.live_budget_extension]
- [ ] [serial] Refresh the self-check baseline if needed and run the full flake check. r[octet.aeneas_consumer_contract.live_budget_extension]

## Verification Coverage

- `Scenario: Cold Lean closure completes within the extended budget` -> `mantle-aeneas-consumer-job`
