# Tasks: Refresh reviewed object-corpus evidence for the canary source

- [x] [serial] Re-freeze the candidate/reviewed manifests and verifier anchors for the selected `lib.rs` byte change. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Run the reviewed-source verifier and object-corpus gate. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Run the full flake check; record any unrelated remaining timeout. r[octet.object_corpus.reviewed_refresh]
  - Full flake check all checks passed; evidence: `evidence/full-flake-check.json`.

## Verification Coverage

- `Scenario: Refreshed source is admitted` -> reviewed-source verifier
- `Scenario: Object-corpus gates replay the refreshed references` -> `object-corpus-gate`
- `Scenario: Independent audit is not claimed` -> mechanical-refresh scope
