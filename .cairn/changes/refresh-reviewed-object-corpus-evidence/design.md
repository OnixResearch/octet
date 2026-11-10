# Design: Refresh reviewed object-corpus evidence

## Context

`cargo-octet/tests/fixtures/object-corpus/source-token-v2/` contains a frozen
candidate/reviewed source snapshot and four reference artifacts. The source
manifest and the verifier's anchors are deliberately frozen; any source change
requires a reviewed refresh.

The integrated source has changed since `b65337f1`. The selected 37 paths and
three reviewed modules still exist, but their bytes and the resulting receipts
differ.

## Decisions

- Refresh both the candidate and reviewed manifests together so the verifier's
  equality checks remain meaningful.
- Regenerate `internal-corpus-receipt.json`,
  `expanded-internal-corpus-receipt.json`, and
  `expanded-internal-corpus-caveat-taxonomy.json` with the checked
  `cargo-octet object corpus receipt` / `corpus caveats` commands.
- Update `CANDIDATE_B3`, `REVIEWED_B3`, and the reference counts in
  `tools/verify-reviewed-object-source.py`.
- Restore `reviewedObjectSource = ./.` so the gates compare the current tree
  against the refreshed manifest.
- Record the refresh as mechanical. The approval scope and independent-audit
  note explicitly state that the original audit was not repeated.

## Verification

- `python3 tools/verify-reviewed-object-source.py --repo . --valence-module
  <pinned valence module>` admits the refreshed source.
- `nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism`
  passes.
- `nix build .#checks.x86_64-linux.object-corpus-caveat-taxonomy-gate` passes.
- The full `nix flake check -L` is rerun.

## Non-claims

A passing refresh proves only that the frozen evidence matches the selected
current source bytes and regenerated references. It does not prove source
correctness, whole-program behavior, semantic equivalence, or release
eligibility, and it is not a new independent owner audit.
