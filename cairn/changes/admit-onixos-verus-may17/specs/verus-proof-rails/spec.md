# Verus Proof Rails Specification Delta

## ADDED Requirements

### Requirement: OnixOS May 17 admission follows local candidate execution [r[octet.verus_proof_rails.onixos_may17_compatibility]]

Octet MUST promote the `onixos-lifecycle` compatibility row only from an OnixOS-owned candidate receipt that binds the exact provider revision, source, package identities, execution transcript, and partial-chain non-claims.

#### Scenario: Candidate execution authorizes the row update

- GIVEN the OnixOS candidate receipt reports four verified obligations, zero errors, no timeout, and promotion eligibility
- WHEN Octet updates its consumer inventory
- THEN only `onixos-lifecycle` MAY become `native` at May 17
- AND Aspen and unrelated consumers MUST remain unchanged

#### Scenario: Candidate identity or result is invalid

- GIVEN the receipt hash, provider revision, result count, timeout status, or consumer-row boundary differs
- WHEN Octet evaluates admission
- THEN the OnixOS row MUST remain `local-pin`
- AND package availability MUST NOT substitute for downstream execution

#### Scenario: Authority does not transfer

- GIVEN Octet publishes a native compatibility row
- WHEN OnixOS consumes it
- THEN OnixOS MUST retain proof-source and receipt authority
- AND the row MUST NOT claim verifier soundness, model completeness, physical readiness, whole-chain completion, or release eligibility
