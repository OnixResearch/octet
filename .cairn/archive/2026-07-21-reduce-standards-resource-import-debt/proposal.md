# Proposal

## Why

The canonical self-check still accepts 1,112 `non_trait_imports` warning occurrences represented by 370 stable identities after two monotonic reductions. The unique private `check_resources` import in the pure standards runner has one call site, and the owner file contains only import findings. An old-baseline run proves this candidate removes exactly one stable identity without the collateral identity churn observed in previously falsified whole-file cohorts.

## What Changes

- Replace the private `check_resources` import with the explicit `crate::resource::check_resources` owner path at its single runner call site.
- Require the accepted 1,818-identity baseline to report exactly stale ID `b3:3965b4dd27194e41`, zero new IDs, and warning-only exit `0` before refresh.
- Regenerate the baseline as an exact removals-only set, lower canonical counts to 2,667 warnings and 1,817 identities, and preserve permanent positive/negative ratchet semantics.
- Record bounded portfolio, standards behavior, regression, traceability, source-identity, and compatible-system evidence before deterministic archive.

## Impact

- **Files**: `crates/octet-standards/src/runner.rs`, the self-check baseline and documentation, Nix assertions, Cairn policy/traceability sources, and source-derived fixtures only if the repository-owned freshness rail requires them.
- **Testing**: focused positive and negative resource checks, complete standards tests, old/refreshed canonical runs, exact baseline-set comparison, permanent self-check and traceability gates, policy freshness, object-corpus determinism, formatting, Clippy, Nextest, and full compatible-system flake validation.
