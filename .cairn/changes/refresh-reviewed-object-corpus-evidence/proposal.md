# Proposal: Refresh reviewed object-corpus evidence

## Why

The reviewed v2 object-corpus evidence was frozen at commit `b65337f1`. The
integrated source now includes the benchmark, debug-map, Kamacite, durability,
boolean, merge-rail, and proof-candidate modules. The object-corpus gates
compare the current source with the frozen references and fail because the
current source produces 1,579 expanded objects instead of the reviewed 1,258.

## What Changes

- Re-freeze the candidate and reviewed source manifests to the current 37
  selected source paths and three reviewed Octet modules.
- Regenerate the internal and expanded corpus receipts and the caveat taxonomy
  from the current selected sources.
- Update the verifier anchors, reference counts, and reviewed-source input.
- Record the re-freeze as a mechanical refresh, not a repeated independent
  audit.

## Non-claims

The re-freeze does not prove source correctness, semantic equivalence, or
release eligibility. The original independent audit covered the earlier frozen
snapshot and was not repeated. The object corpus remains bounded source-token
linkage and provenance evidence.
