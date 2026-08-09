## Why

Octet owns the checked Rust path, standards checks, policy traceability, and source-shape evidence rails. ast-grep provides a fast syntax-aware layer that can catch reviewable source shapes, prototype codemods, and produce structured candidate findings before a rule is promoted into Dylint, proof rails, or stronger Octet checks. Octet should own the stack-wide ast-grep rule-pack discipline while preserving clear non-claim boundaries.

## What Changes

- Define an Octet ast-grep structural rail profile for syntax-only search, lint, and codemod candidates.
- Add repo-owned rule bundle layout, positive and negative rule tests, severity posture, and promotion criteria from advisory inventory to blocking check.
- Record ast-grep version, rule bundle BLAKE3 identity, input path scope, JSON/SARIF findings, and non-claims in Octet evidence artifacts.
- Keep generated findings as candidate evidence until an Octet-native lint, proof rail, or explicit policy gate promotes them.

## Impact

- **Surfaces**: standards checks, evidence collect rails, source-shape lint catalog, migration sessions, proof-card/non-claim reports, Valence export adapters.
- **Non-claims**: ast-grep findings do not prove Rust type correctness, semantic correctness, panic freedom, memory safety, proof discharge, release eligibility, or whole-program safety.
- **Validation**: ast-grep rule tests, focused scan fixtures, Octet artifact verification, Cairn gates, and existing focused Octet checks.
