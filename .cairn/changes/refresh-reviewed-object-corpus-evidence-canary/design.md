# Design: Refresh reviewed object-corpus evidence for the canary source

## Context

`cargo-octet/tests/fixtures/object-corpus/source-token-v2/` freezes the reviewed
source manifests and four reference artifacts. The canary rail changed the
selected `crates/octet-standards/src/lib.rs` source bytes.

## Decisions

- Update the selected `lib.rs` hash in both manifests.
- Update the reviewed manifest's candidate binding and mechanical-refresh scope.
- Update `CANDIDATE_B3`, `REVIEWED_B3`, and keep the reference counts and file
  hashes unchanged because the reference artifacts are unchanged.
- Record the mechanical refresh; the original independent audit was not
  repeated.

## Verification

- `python3 tools/verify-reviewed-object-source.py --repo . --valence-module
  <pinned valence module>` admits the refreshed manifests.
- `nix build .#checks.x86_64-linux.object-corpus-gate` passes.
- The full flake check is attempted; any remaining unrelated timeout is
  recorded explicitly.

## Non-claims

The refresh does not prove source correctness, whole-program behavior, semantic
equivalence, or release eligibility. It is not a new independent owner audit.
