# Aeneas Consumer Contract Delta

## ADDED Requirements

### Requirement: External-consumer budget covers the pinned Lean closure

r[octet.aeneas_consumer_contract.live_budget] The external-consumer profile
MUST set a bounded proof timeout that covers the pinned Aeneas/Lean closure's
cold Mathlib build on the supported host, and the selected Aeneas rail, the
consumer profile, and the provider authority maximum MUST agree on that bound,
while retaining the existing process, environment-allowlist, output, and
network-denial bounds.

#### Scenario: Cold Lean closure completes within the consumer budget

- GIVEN the packaged Aeneas provider and the external consumer fixture
- WHEN the consumer proof runs from a cold pinned Mathlib build
- THEN the Lean stage MUST complete within the profile timeout
- AND the consumer outcome MUST be `supported`.
