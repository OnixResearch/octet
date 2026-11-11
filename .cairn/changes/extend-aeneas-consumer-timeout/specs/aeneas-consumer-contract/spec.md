# Aeneas Consumer Contract Delta

## ADDED Requirements

### Requirement: Extended live budget covers the cold Lean closure

r[octet.aeneas_consumer_contract.live_budget_extension] When the pinned cold
Mathlib build exceeds the current external-consumer budget on the supported
host, the external-consumer profile MUST raise its bounded proof timeout and
update the provider authority maximum, selected Aeneas rail timeout, live-test
budgets, generated profile identity, and expected receipt together, while
retaining the process, environment, output, and network-denial bounds.

#### Scenario: Cold Lean closure completes within the extended budget

- GIVEN the packaged Aeneas provider and the external consumer fixture
- WHEN the consumer proof runs from a cold pinned Mathlib build
- THEN the Lean stage MUST complete within the extended profile timeout
- AND the consumer outcome MUST be `supported`.
