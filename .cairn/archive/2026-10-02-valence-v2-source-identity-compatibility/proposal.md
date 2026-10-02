## Why

The accepted `.cairn/specs/valence-project-extraction/spec.md` records a historical Phase 2 Function Object v1 extraction contract. Octet now selects an independently reviewed upstream Valence commit that admits `octet-source-token-v2`; existing source-backed commands construct v2 identities, while the unchanged v1 corpus receipt is still readable but no longer matches default source replay. A current native compatibility contract must record those versioned facts without rewriting accepted Phase 2 text or archived evidence.

## What Changes

- Add current v2 normalization, historical v1 artifact-read/stale-source-replay, source-bound independently reviewed corpus/aggregate, and bounded lifecycle requirements to the accepted Valence extraction spec through native Cairn sync.
- Require the existing named source-bound checks to fail on stale source bytes, missing owner-reviewed references, or false boundary claims; two matching freshly generated receipts are not approval.
- Leave the historical Phase 2 requirements, archives, v1 fixture bytes, owner-frozen H2 references, and accepted evidence unchanged. H1/H2 owner decisions and independently audited manifest bind only their expressly reviewed scope; no new review signature or correctness claim is implied.

## Impact

- **Files**: This new `.cairn/changes/valence-v2-source-identity-compatibility/` package; native sync may append v2 requirements to `.cairn/specs/valence-project-extraction/spec.md`. A separate reviewed Octet OpenSpec delta exists under `openspec/changes/source-token-v2-function-object-cutover/` until supported promotion.
- **Testing**: Prior independently saved OpenSpec V1–V17 source-backed and named-rail transcripts, exact pinned Cairn policy/schema admission and native proposal/design/tasks gates plus validation, immutable historical file comparison, and strict OpenSpec validation before any OpenSpec archive.
