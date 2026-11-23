# Tasks: Admit the owner-approved object-corpus refresh for PR #38

## Phase 1: Change package

- [x] [serial] Record why PR #38's `cli.rs` flags break the reviewed-source admission and what the mechanical refresh covers. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Specify the manifest rebind and separate historical audit and current approval. r[octet.object_corpus.reviewed_refresh]

## Phase 2: Mechanical refresh

- [x] [serial] Set `cargo-octet/src/cli.rs`'s hash to the branch bytes in the candidate and reviewed manifests, rebind the reviewed `candidate_manifest`, and add the top-level mechanical-refresh note. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Refresh the verifier's `CANDIDATE_B3` and `REVIEWED_B3` anchors and bind the unchanged audit envelope to its original audited candidate identity. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Preserve the audit envelope, original producer/source binding, historical approval, matching fixed reference and Valence binding. r[octet.object_corpus.reviewed_refresh]

## Phase 3: Replay evidence

- [x] [serial] Reproduce all four pre-refresh references byte-for-byte on base `3ddd21ef` with the immutable `0.1.1-rc.1` producer. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Replay the internal, expanded, taxonomy and fixed references against the branch and record exactly what differs. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Re-capture the three affected references with the baseline-proven producer, update the manifest rows and the verifier counts, and recompute both manifest anchors. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Run the test filter for every `mkObjectRail` and record the local test results. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Record the direct owner decision and require the pinned record and exact tuple in the verifier. r[octet.object_corpus.reviewed_refresh]
- [x] [serial] Verify exact refresh admission and rejection with no usable PR #38 owner decision. r[octet.object_corpus.reviewed_refresh]

## Verification Coverage

- `Scenario: Refreshed source is admitted` -> `evidence/reviewed-source-verifier.txt`
- `Scenario: An unauthorised refresh is refused` -> `evidence/reviewed-source-verifier.txt`
- `Scenario: Re-captured references are baseline-proven` -> `evidence/reference-recapture.txt`
- `Scenario: Object rails replay the approved tuple` -> `evidence/object-rail-matrix.txt` (local test bodies; CI required separately)
- `Scenario: Unchanged approval record keeps its exact bytes` -> verifier historical byte locks and tamper tests
- `Scenario: Independent audit is not claimed` -> the pinned PR #38 decision and this package's non-claims
