# Proposal

## Why

The canonical self-check still accepts 1,111 `non_trait_imports` warning occurrences represented by 369 stable identities after three monotonic reductions. The unique private `CheckReport` import in the pure standards runner has one return-type use and one construction use, and the owner file still contains only import findings. The checked-in baseline identifies this candidate as stable ID `b3:b918c07b0ac7abe7`; an old-baseline run must prove that qualifying both use sites removes exactly that identity without changing report behavior or introducing collateral identity churn.

## What Changes

- Replace the private `CheckReport` import with explicit `crate::report::CheckReport` owner paths in the `run_checks` return type and report construction.
- Require the accepted 1,817-identity baseline to report exactly stale ID `b3:b918c07b0ac7abe7`, zero new IDs, and warning-only exit `0` before refresh.
- Regenerate the baseline as an exact removals-only set, lower canonical counts to 2,666 warnings and 1,816 identities, and preserve permanent positive/negative ratchet semantics.
- Record bounded portfolio, report behavior, regression, traceability, source-identity, and compatible-system evidence before deterministic archive.

## Impact

- **Files**: `crates/octet-standards/src/runner.rs`, the self-check baseline and documentation, Nix assertions, Cairn policy/traceability sources, and source-derived fixtures only if the repository-owned freshness rail requires them.
- **Testing**: focused positive and negative report behavior tests, complete standards tests, old/refreshed canonical runs, exact baseline-set comparison, permanent self-check and traceability gates, policy freshness, object-corpus determinism, formatting, Clippy, Nextest, and full compatible-system flake validation.
