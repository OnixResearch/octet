# valence-core

`valence-core` contains the pure deterministic core logic that Octet uses for bounded Valence evidence surfaces.

Current in-repository surfaces:

- evidence-index render/check logic;
- Function Object v1 DTO, canonical hash, caveat summary, and pure-cache eligibility construction.

## Boundary

This crate is intentionally an in-repository readiness seam, not a standalone release cutover. It does not own CLI parsing, filesystem access, source scanning, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, process exits, repository tags, or package publication. Those remain Octet-owned until a future reviewed cutover changes the contract.

## Evidence limits

The crate supports bounded local evidence replay for selected Octet/Valence surfaces. It does not claim behavioral correctness, semantic equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.
