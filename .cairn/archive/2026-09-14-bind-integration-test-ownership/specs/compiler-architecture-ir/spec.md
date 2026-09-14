## ADDED Requirements

### Requirement: Normalized facts retain compilation-unit membership
r[octet.architecture.unit_membership]

Normalized Architecture IR MUST retain every validated unit's fact and unknown memberships before deduplication.
Membership MUST be canonical, required, covered by the IR identity, and checked against all declared units, facts, and unknowns.
Missing or noncanonical unit rows, duplicate or unsorted member IDs, orphan IDs, and subjects without any membership MUST fail verification.
Recomputing the enclosing identity MUST NOT change that rejection.
A change to any shared membership edge MUST change that identity. Structural verification MUST NOT claim to authenticate a structurally possible resealed relation.
A v1 IR without memberships MUST NOT acquire invented ownership.

#### Scenario: Shared facts retain both observing units

- GIVEN two complete validated shards that share facts and unknowns
- WHEN normalization deduplicates those values
- THEN both unit memberships MUST remain and output MUST be invariant under shard ordering

#### Scenario: Malformed membership is resealed

- GIVEN a normalized IR with a missing unit row, noncanonical row or member order, duplicate or orphan IDs, or an unreferenced subject
- WHEN its enclosing identity is recomputed and verification runs
- THEN verification MUST reject the malformed relation

#### Scenario: Legacy or incomplete evidence is supplied

- GIVEN legacy IR without memberships or a missing selected compiler shard
- WHEN decoding or normalization runs
- THEN the input MUST remain unsupported or incomplete without ownership fallback
