## Context

The accepted `valence-project-extraction` Phase 5 requirements already call for a standalone dependency manifest, compatibility fixtures, shell/core separation, consumer migration docs, and bounded validation. The remaining risk is operational: both repositories still publish local package metadata as `valence-core` with the same version while their module sets differ, and downstream resolution can select the older implementation.

## Decisions

### Pin one canonical source

Octet records the standalone Valence source coordinate and exact immutable revision in a reviewed dependency manifest. Local development overrides are explicit, non-release inputs and cannot alter release lock evidence. The lockfile and manifest must agree on source and revision.

The selected revision must include the archived Valence corrected Preserves integrity profile. Octet does not accept a branch-only, tag-only, or floating SSH dependency as release evidence.

### Eliminate duplicate package identity

The Octet-hosted crate is either removed after adapters migrate or renamed to an explicitly legacy compatibility package whose name and version cannot be confused with standalone `valence-core`. A package graph check rejects more than one semantic provider for canonical Valence operations and rejects two different source identities sharing the canonical package name/version tuple.

### Preserve the functional boundary

Standalone Valence owns pure Evidence IR DTOs, canonicalization, claim policy, and provenance semantics. Octet retains source scanning, lint execution, proof-rail orchestration, filesystem access, process execution, stdout/stderr, and publication. Compatibility shims contain no forked semantic implementation.

### Validate compatibility with bounded fixtures

Positive fixtures cover Function Object, evidence-index, and evidence-codebase-index compatibility declared by the accepted Phase 5 spec. Negative fixtures cover wrong revision, stale local copy, omitted module row, altered non-claims, lock drift, duplicate package identity, and accidental fallback to the hosted implementation.

### Cross-repository ordering

Implementation requires the archived Valence `harden-preserves-integrity-boundaries` receipt. Aspen's `pin-canonical-evidence-dependencies` package waits for this package's archive receipt. These prerequisites remain explicit in lifecycle artifacts until Cairn cross-repository dependency metadata is available.

### CI scope

The checked-in workflow runs only `nix flake check`. Focused compatibility and package-graph tests remain required implementation evidence outside the minimal CI command scope.

## Risks / Trade-offs

- Renaming or removing the hosted package may break undeclared consumers; migration docs and a bounded shim reduce the risk without preserving ambiguous identity.
- Fixture parity does not prove general semantic equivalence; receipts keep that non-claim explicit.
- Pin updates require reviewed lock and compatibility evidence, but that is necessary for reproducibility.
