# Proposal: Refresh reviewed object-corpus evidence for the canary source

## Why

The recorded-only Vermilion canary rail adds a module declaration to
`crates/octet-standards/src/lib.rs`, a selected reviewed object-corpus source.
The frozen candidate/reviewed manifests and verifier anchors no longer match
the selected source bytes.

## What Changes

- Re-freeze the candidate and reviewed source manifests for the selected
  `lib.rs` byte change.
- Update the verifier anchors and mechanical-refresh scope.
- Keep the four reference artifacts, their counts, the 37-path/three-module
  selection, and the Valence module/revision binding unchanged.
- Record the refresh as mechanical; the original independent audit was not
  repeated.

## Non-claims

The refresh proves only that the frozen evidence matches the selected current
source bytes and references. It does not prove source correctness, semantic
equivalence, or release eligibility, and it is not a new independent audit.
