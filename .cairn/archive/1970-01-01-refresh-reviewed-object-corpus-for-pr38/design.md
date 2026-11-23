# Design: Bind the PR #38 object-corpus refresh to its owner decision

## Context

`cargo-octet/tests/fixtures/object-corpus/source-token-v2/` freezes the reviewed
source manifests, the four reference artifacts, the approved audit envelope and
the original producer/source binding. `tools/verify-reviewed-object-source.py`
admits only the exact byte tuple it names. The canary refresh
(`.cairn/archive/2026-11-11-refresh-reviewed-object-corpus-evidence-canary/`,
`20a19385`) is the precedent for a mechanical refresh after one selected-source
byte change; `1b594c47` added the exact owner-approval block that the verifier
now compares literally.

## Decisions

### Decision: Refresh only the changed selected source

**Choice:** In both manifests, set only `cargo-octet/src/cli.rs`'s hash to the
branch bytes (`d181d2d9…`). Every other selected-source hash, the selections,
three selected Octet modules, test snapshot and reference paths are unchanged.

**Rationale:** Exactly one selected source changed, so the refresh must not
carry any other byte change.

### Decision: Rebind the candidate file and the verifier anchors

**Choice:** Set `reviewed.candidate_manifest.blake3` to the refreshed candidate
digest. Pin `CANDIDATE_B3` and `REVIEWED_B3` from the exact tuple submitted to
the owner; do not edit either file after recording the decision.

**Rationale:** A decision on two already frozen manifests avoids circular
hashing or treating a changed manifest as if it had been approved.

### Decision: Keep the audit envelope bound to its original audited candidate

**Choice:** The approved audit envelope, the producer/source binding and the
approval block stay byte-identical. The verifier keeps an
`AUDIT_CANDIDATE_B3` constant holding the original audited candidate identity
(`be3647f9…`) and requires the unchanged envelope's `canonical_candidate` to
match that constant instead of the refreshed `CANDIDATE_B3`.

**Rationale:** The audit envelope records the immutable producer's pre-freeze
capture, including `source-snapshot/cargo-octet/src/cli.rs` with the old bytes.
Editing it would destroy the historical audit record. The live tuple and the
sealed historical records are checked separately.

### Decision: Record the mechanical refresh outside the approval block

**Choice:** The reviewed manifest's `mechanical_refresh` records the changed
path, old/new hashes and affected references. Its `owner_decision` text
describes the state at preparation time, not the current authority. The
historical `approval` block keeps its exact bytes.

**Rationale:** The mapping identifies evidence deltas but grants no authority.

### Decision: Re-capture the three affected references with the recorded producer

**Choice:** Re-run the three corpus capture commands with the immutable
`cargo-octet 0.1.1-rc.1` producer
(`/nix/store/1caf2mjpb6bs33w2s05lbmhwladysqn3-cargo-octet-0.1.1-rc.1/bin/cargo-octet`)
and replace `internal-corpus-receipt.json`,
`expanded-internal-corpus-receipt.json` and
`expanded-internal-corpus-caveat-taxonomy.json` with its output. Before writing,
prove on `origin/main` that this producer reproduces all four earlier references
byte-for-byte. Leave `positive-v2-receipt.json` untouched.

**Rationale:** The owner needs a complete, exact tuple to decide on, and three
references provably cannot stand unchanged. Re-capturing with a producer that
reproduces the audited references first keeps the method honest; the two new
objects (`RawLogRetention::as_str`, `RawLogRetention::parse`) explain the count
move exactly.

### Decision: Bind sealed records to the audited identities

**Choice:** Compare sealed source and reference identities with the previous
hashes in the pinned refresh note. Separately check current bytes and counts
against both approved manifests.

**Rationale:** The audit envelope must not be rewritten or construed to
describe unaudited source/reference bytes.

### Decision: Require a separate pinned refresh approval

**Choice:** Record the exact interactive submission and authorized identities
in `cargo-octet/tests/fixtures/object-corpus/source-token-v2/pr38-object-refresh-owner-approval.txt`.
The verifier requires the regular file at that fixed path, its pinned BLAKE3,
the affirmative reply, and both exact manifest digests. No fallback to the
historical approval or to the manifest note is allowed.

**Rationale:** The frozen manifests can remain byte-for-byte what the owner
reviewed; the decision file is outside the lifecycle archive move.

## Risks / Trade-offs

- The owner approved the recaptured bytes, not a new audit of them. The
  verifier and its byte locks are in the same repository; they detect drift
  but do not authenticate who changed the verifier or signed a decision.

## Verification

- `python3 tools/verify-reviewed-object-source.py --repo . --valence-module
  <pinned-valence-module>` admits the exact refreshed tuple.
- `python3 tools/test-verify-reviewed-object-source.py --repo . --valence-module
  <pinned-valence-module>` rejects missing, historical, altered and symlinked
  refresh records in otherwise matching trees, in addition to the old probes.
- `evidence/reference-recapture.txt` records byte-identical baseline replay;
  `evidence/object-rail-matrix.txt` records local test filters, not the Nix
  aggregate or GitHub CI result.

## Non-claims

This admission is not an independent audit, signature, correctness proof,
semantic equivalence or release qualification.
