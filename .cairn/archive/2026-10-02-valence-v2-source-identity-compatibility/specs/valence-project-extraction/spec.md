# Valence v2 Source Identity Compatibility Delta

## Purpose

Define the selected source-token-v2 Function Object contract alongside, not in place of, Octet's accepted historical Phase 2 v1 extraction contract.

## ADDED Requirements

### Requirement: Selected Function Object source identity is versioned

r[octet.valence_project_extraction.v2.normalization]

Octet MUST construct current source-backed Function Objects with the independently reviewed Valence identity `octet-source-token-v2`, bound to one exact Cargo/Nix/Valence cutover revision. It MUST NOT label corrected v2 source as `octet-source-token-v1` or claim correctness, semantic equivalence, or pure-cache eligibility from a source-token hash.

#### Scenario: Quoted URL and executable edits change identity

- GIVEN the reviewed fixed Rust function containing a quoted URL before executable code
- WHEN the URL literal or executable return expression changes without changing the function selection
- THEN the v2 object identity MUST change; unchanged-token ordinary trivia MUST preserve token identity, while a changed source byte can still invalidate source-bound review.

#### Scenario: Unreviewed normalization is refused

- GIVEN a Valence pin that cannot admit the requested v2 normalization identity
- WHEN source-backed Function Object construction is requested
- THEN construction MUST fail closed rather than present corrected bytes under the v1 label.

### Requirement: Historical v1 artifacts are readable but not rebound

r[octet.valence_project_extraction.v2.legacy_compatibility]

Octet MUST preserve the accepted Phase 2 Function Object v1 canonical payload, immutable historical v1 receipts and negative compatibility fixtures, and artifact-only v1 reading. The default v2 source-replay path MUST compare versioned identities rather than rewrite an old receipt, marker, pointer, policy, archive, or accepted Phase 2 record to obtain a match.

#### Scenario: Historical v1 receipt mismatches default v2 replay

- GIVEN the unchanged checked-in v1 receipt and its checked-in source
- WHEN `object corpus check` constructs source-backed v2 records
- THEN the valid v1 receipt MUST report nonzero `status=mismatch` on `object_set_hash`, while artifact-only v1 receipt/object readers remain available and invalid receipt shapes still report typed errors.

#### Scenario: Selected v2 receipt is independent

- GIVEN the separately reviewed v2 fixed-source receipt and its exact source-byte binding
- WHEN the same source is checked
- THEN the v2 receipt MUST replay as `status=match` with false correctness and semantic-equivalence boundaries; source or boundary tampering MUST fail closed.

### Requirement: Native v2 lifecycle leaves Phase 2 history intact

r[octet.valence_project_extraction.v2.lifecycle_boundary]

This native Cairn delta MUST add current v2 requirements without mutating accepted Phase 2 v1 requirements, prior accepted receipts, or historical Cairn archives. The active OpenSpec v2 change MAY be promoted only through its supported archive/spec-sync workflow after its own strict validation and native Cairn validation; it MUST NOT be promoted by direct hand-editing of the canonical OpenSpec spec.

#### Scenario: Versioned delta is validated without migrating history

- GIVEN the existing accepted Valence extraction spec and archived Phase 2 change
- WHEN the native v2 change is created, gated, validated, synced, and archived
- THEN the accepted Phase 2 v1 requirement text and archived bytes MUST remain unchanged while new v2 requirements become distinguishable in the accepted spec.

## MODIFIED Requirements

### Requirement: Current v2 corpus evidence is source-bound and bounded

r[octet.valence_project_extraction.v2.reviewed_corpus]

Existing named internal, expanded, caveat-taxonomy, fixture-freshness, supplement, fresh-build, and aggregate rails MUST admit the selected v2 source/reference manifest before PASS. The approved binding MUST identify exact selected source paths and source-byte hashes, all four independently saved v2 reference hashes, the reviewed Valence revision `7546a78b4eeef8686c6837e187aa12420f4f03bb` and its `function_object.rs` BLAKE3 `11f0f4233aaf3153966f6aa37de570c2dcd0e75335591934461ae2b25ab3ecb1`, and the scoped owner review. The immutable historical candidate manifest BLAKE3 `6d79a8b09c4e7dd20236f4a141c01d6c18b793df7aad93e8ee4ec1d7fb83b221` remains `PENDING`; the independently reviewed manifest BLAKE3 is `2f420d0464e67a8b6ae518e7215eefd1ba8afc47d1849c07afedf284f196da29`. An independent reviewer MUST compare those frozen anchors outside the PR when authenticating H2: an in-repository verifier and its constants can be coedited and do not themselves cryptographically authenticate owner approval. Any changed source/reference binding requires separate owner review, never automatic refresh. A `PENDING` candidate, a token-stable source-byte edit, or two self-produced matching receipts MUST NOT substitute for the approved references; compiled producer changes outside the frozen H2 scope require separate review.

#### Scenario: Missing approval or stale source denies a named rail

- GIVEN a changed selected source byte, absent/altered saved reference, or absent scoped approval
- WHEN a named source-backed v2 rail runs
- THEN it MUST fail before a PASS even if regenerated object sets or two independently built binaries agree; unresolved caveats and both false claim boundaries MUST remain explicit.

#### Scenario: Fresh binaries replay both approved corpus references

- GIVEN two independently built selected Octet binaries and approved 18-path internal and 36-path expanded reference receipts
- WHEN the existing fresh-build and aggregate rails run
- THEN each binary's internal and expanded receipts MUST match the other binary and the corresponding independently reviewed saved reference before named PASS; no correctness, exhaustive extraction, cross-toolchain, or cross-machine claim follows.
