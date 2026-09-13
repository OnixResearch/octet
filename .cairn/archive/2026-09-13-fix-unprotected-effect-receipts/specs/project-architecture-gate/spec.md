# Authority-scoped architecture receipt validation

## ADDED Requirements

### Requirement: Authorization identities follow protected effect declarations
r[octet.project_architecture_gate.authority_scoped_receipt_ids]

Receipt verification MUST require nonempty authorization fact and type identity arrays when at least one outbound-effect classification is protected.
An unprotected-only effect list MUST NOT require protected authorization identities.
The authority discriminator MUST control this requirement, independently of declaration order and optional protocol payload presence.
Receipt identity, schema, evidence tier, parent bindings, required non-claims, and architecture findings MUST retain their existing checks.
Successful self-verification MUST NOT promote an unprotected effect to authorized execution or establish a passing architecture phase.

#### Scenario: Unprotected receipt has no protected identities

- GIVEN a complete valid architecture evaluation with only unprotected outbound effects
- WHEN the producer emits empty authorization identity arrays and the receipt is serialized and read back
- THEN receipt self-verification MUST succeed and preserve the complete classification and non-claims

#### Scenario: Protected requirement survives a mixed effect list

- GIVEN a protected or mixed effect list with a current receipt identity
- WHEN either authorization identity array is empty
- THEN verification MUST reject the receipt regardless of declaration order or optional protocol payload presence

#### Scenario: Unprotected classification does not bypass other checks

- GIVEN separate cases with a tampered receipt, a missing required non-claim, or an actual unclassified outbound effect
- WHEN receipt verification or architecture evaluation checks the affected boundary
- THEN the corresponding identity, claim-boundary, or effect-classification check MUST reject that case
