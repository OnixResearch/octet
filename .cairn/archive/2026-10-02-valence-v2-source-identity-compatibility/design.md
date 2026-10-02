## Context

The reviewed external Valence revision `7546a78b4eeef8686c6837e187aa12420f4f03bb` constructs source-token-v2 objects under the existing typed Function Object v1 record schema; it does not redefine historical source-token-v1 hashes. The H2 reviewed manifest binds exact bytes for 37 selected Rust sources and four v2 reference artifacts, with separate frozen candidate and reviewed BLAKE3 anchors. Existing Cairn Phase 2 requirements and archive record the former v1 extraction step and must retain their bytes. Installed newer Cairn requires an incompatible `outcome-machine` profile; Octet's pinned Cairn `695124d459574ba7aeba6097310d237f393c243c` instead admits the unchanged authored `spec-driven` policy.

## Decisions

### Decision: Add current requirements without rewriting Phase 2

**Choice:** Use native `cairn change create --profile spec-driven` with the generated Octet policy, add distinct `octet.valence_project_extraction.v2.*` requirements in the same spec family, and sync only after native gates/validation pass. Preserve all Phase 2 v1 requirement IDs/text and historical archives unchanged; no policy migration or new workflow profile is needed.

**Rationale:** The Phase 2 wording describes a real earlier v1 extraction and the v2 review explicitly authorizes a replacement current source-replay behavior, not retroactive mutation of accepted evidence. Versioned requirement IDs make both eras inspectable.

### Decision: Keep independent source approval separate from local replay

**Choice:** Bind selected current v2 source paths, byte hashes, exact Valence SHA `7546a78b4eeef8686c6837e187aa12420f4f03bb`, its `function_object.rs` BLAKE3 `11f0f4233aaf3153966f6aa37de570c2dcd0e75335591934461ae2b25ab3ecb1`, four saved references, and scoped H2 approval before a named rail can pass. The frozen historical `PENDING` candidate manifest BLAKE3 is `6d79a8b09c4e7dd20236f4a141c01d6c18b793df7aad93e8ee4ec1d7fb83b221`; the separate approved reviewed manifest BLAKE3 is `2f420d0464e67a8b6ae518e7215eefd1ba8afc47d1849c07afedf284f196da29`. Compare both anchors out of PR before attributing H2 owner approval; in-PR verifier PASS is no signature. An intentional source/reference refresh requires separate independent review. Keep correctness and semantic-equivalence boundaries false; historical v1 artifact reads do not imply v1 source replay under v2 matches.

**Rationale:** A new non-doc comment can preserve the token object-set hash while changing reviewed source bytes. Two fresh binaries agreeing with each other cannot substitute for the independently saved reviewed receipts.

### Decision: Preserve policy authority and provenance

**Choice:** Use the exact locked Cairn SHA via authenticated `git+ssh` transport; its default Nickel policy source is byte-identical to Octet's `cairn-policy/upstream-default.ncl`. Leave authored `cairn-policy/default.ncl`, generated JSON, and historical archives unchanged. `cairn archive` handles only the newly validated native change; separate `openspec archive` handles canonical OpenSpec promotion only after all tasks/checkpoints pass.

**Rationale:** The private GitHub archive endpoint returns HTTP 404, while SSH fetch of the SAME SHA succeeds. A new `outcome-machine` policy grant merely to satisfy an unrelated newer installed CLI would broaden authorization without source authority.

## Risks / Trade-offs

- The verifier implementation and its expected digests share a PR-controlled tree: a malicious coordinated edit could create an in-PR PASS. External review must compare frozen candidate/reviewed BLAKE3 anchors outside the PR.
- H2 binds selected sources and saved references, not arbitrary later producer edits in `valence_core_shim.rs` or `object_core.rs`; re-review changed producer behavior separately.
- Native sync or OpenSpec archive could alter historical accepted text or evidence paths. Inspect their actual mutations and stop instead of accepting any historical Phase 2/archive rewrite. After supported OpenSpec archive, replace any placeholder purpose and dead active evidence paths; never direct-edit the canonical spec to implement the delta.
