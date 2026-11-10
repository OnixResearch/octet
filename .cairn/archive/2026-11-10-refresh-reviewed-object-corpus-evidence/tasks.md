# Tasks: Refresh reviewed object-corpus evidence

- [x] [serial] Update the candidate and reviewed source manifests to the current 37 selected paths and three reviewed modules. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Regenerate the internal and expanded corpus receipts and caveat taxonomy. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Update the verifier anchors and reference counts. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Restore the reviewed source input to the current tree. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Run the reviewed-source verifier, the fresh-build and caveat-taxonomy gates, and the full flake check. See `evidence/verification.md` for the focused gates; the full flake check is rerun separately. r[octet.object_corpus.reviewed_refresh]

## Verification Coverage

- `Scenario: Refreshed source is admitted` -> reviewed-source verifier
- `Scenario: Object-corpus gates replay the refreshed references` -> fresh-build and caveat-taxonomy gates
- `Scenario: Independent audit is not claimed` -> approval scope review
