# Proposal

## Why

The canonical self-check still accepts 1,110 `non_trait_imports` warning occurrences represented by 368 stable identities after four monotonic reductions. The remaining private `PolicyEvaluation` import in the pure standards runner has one helper return-type use and one construction use. The checked-in baseline identifies this candidate as stable ID `b3:c0f0bf1de2e13e66`; an old-baseline run must prove that qualifying both use sites removes exactly that identity without changing policy selection, evaluation ordering, enforcement metadata, failure classification, report behavior, or serialized output.

## What Changes

- Replace the private `PolicyEvaluation` import with explicit `crate::report::PolicyEvaluation` owner paths in the helper return type and construction.
- Require the accepted 1,816-identity baseline to report exactly stale ID `b3:c0f0bf1de2e13e66`, zero new IDs, and warning-only exit `0` before refresh.
- Regenerate the baseline as an exact removals-only set, lower canonical counts to 2,665 warnings and 1,815 identities, and preserve permanent positive/negative ratchet semantics.
- Record bounded portfolio, policy-evaluation behavior, regression, traceability, source-identity, and compatible-system evidence before deterministic archive.

## Impact

- **Files**: `crates/octet-standards/src/runner.rs`, the self-check baseline and documentation, Nix assertions, Cairn policy/traceability sources, and source-derived fixtures only if the repository-owned freshness rail requires them.
- **Testing**: focused positive and negative policy-selection/report tests, complete standards tests, old/refreshed canonical runs, exact baseline-set comparison, permanent self-check and traceability gates, policy freshness, object-corpus determinism, formatting, Clippy, Nextest, and full compatible-system flake validation.
