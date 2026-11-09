# Assurance Profiles Delta

## ADDED Requirements

### Requirement: Storage-fault assurance profile
r[octet.durability_claims.assurance_profile] Octet MUST provide a bounded profile that links exact saved storage-fault producer and Cairn receipt identities without executing the producer.

#### Scenario: Current complete evidence is supplied
- GIVEN current saved receipts with required case coverage and non-claims
- WHEN Octet validates the profile
- THEN it emits a passing bounded assurance result

#### Scenario: Saved campaign receipt is stale
- GIVEN a saved receipt for another source or candidate BLAKE3
- WHEN profile validation runs
- THEN Octet rejects the evidence

#### Scenario: Required negative case is missing
- GIVEN a profile that requires flush-error or uncertain-commit evidence
- WHEN the receipt set omits that case
- THEN Octet rejects the evidence

#### Scenario: Producer is not installed
- GIVEN complete current saved evidence and no producer executable
- WHEN profile validation runs
- THEN validation can complete without starting the producer
