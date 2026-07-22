# Evidence Provenance Traceability Delta

## MODIFIED Requirements

### Requirement: Provenance registry ownership

r[octet.evidence_provenance.registry] The provenance record family MUST be registered in `standards/policies/registry.ncl` under the correct ownership classification.

#### Scenario: Registry covers provenance records

r[octet.evidence_provenance.registry.covered]

- GIVEN Octet emits provenance records after execution
- WHEN artifact manifest freshness validation runs
- THEN the provenance family MUST be covered as Rust-derived unless an explicit non-candidate rationale applies.
