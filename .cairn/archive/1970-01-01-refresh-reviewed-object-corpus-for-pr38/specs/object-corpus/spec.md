# Object Corpus Delta

## MODIFIED Requirements

### Requirement: Reviewed object-corpus evidence is refreshed after source integration

r[octet.object_corpus.reviewed_refresh] The reviewed v2 object-corpus candidate
and reviewed manifests, verifier anchors, and reference counts MUST be refreshed
together when the selected source bytes change, including rail source
integrations such as the measured Vermilion canary and CLI-flag changes on a
selected source. The refresh MUST preserve the source/module selections, the
approved audit envelope, the original producer/source binding, the historical
approval record, and the Valence module/revision binding. It MUST leave matching
reference artifacts unchanged and bind sealed records to the identities they
actually recorded, not to refreshed bytes. When replay differs, the refresh MUST
use a producer that first reproduces all approved references byte-for-byte on
the pre-change tree, and record each changed reference's previous and new BLAKE3
and object count. The verifier MUST require a separate pinned owner decision
binding both refreshed manifests before it admits them. A mechanical refresh
note or a prior approval MUST NOT grant admission to the refreshed tuple.

#### Scenario: Refreshed source is admitted

- GIVEN the exact refreshed manifests and references, the pinned Valence module,
  and the owner's pinned approval of the new tuple
- WHEN the reviewed-source verifier runs
- THEN it MUST admit the refreshed tuple and MUST require the unchanged audit
  envelope to match its original audited candidate identity.

#### Scenario: An unauthorised refresh is refused

- GIVEN otherwise matching refreshed inputs but a missing, changed, symlinked,
  or historical decision in place of the exact new owner approval
- WHEN the reviewed-source verifier runs
- THEN it MUST reject before the mechanical-refresh mapping can be used.

#### Scenario: Re-captured references are baseline-proven

- GIVEN a selected source whose new bytes add object-corpus entries
- WHEN the four saved references are replayed against the refreshed sources
- THEN the refresh MUST first prove the capturing producer reproduces every
  approved reference byte-for-byte on the pre-change tree
- AND it MUST re-capture the affected references and record each one's previous
  and new BLAKE3 and object count for the repository owner.
- AND it MUST NOT rewrite the approved audit envelope or producer/source
  binding to describe the new bytes.

#### Scenario: Object rails replay the approved tuple

- GIVEN the exact approved source and recaptured references
- WHEN an object rail runs
- THEN the nonignored source-backed tests MUST pass on the approved tuple.

#### Scenario: Unchanged approval record keeps its exact bytes

- GIVEN the sealed owner-approval record and the approved audit envelope
- WHEN the mechanical refresh is prepared
- THEN both MUST stay byte-identical and the approval block MUST keep its exact
  scoped fields and evidence digest.

#### Scenario: Independent audit is not claimed

- GIVEN the mechanical refresh
- WHEN the approval scope is reviewed
- THEN it MUST state that the original independent audit was not repeated and
  the new owner approval admits only the selected source/reference identities.
