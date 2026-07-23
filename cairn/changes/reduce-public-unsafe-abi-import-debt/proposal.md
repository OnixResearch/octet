# Proposal

## Why

The canonical self-check still accepts 1,113 `non_trait_imports` warning occurrences represented by 371 stable identities after the first monotonic reduction. The unique private `rustc_abi::ExternAbi` import in `src/safety/public_unsafe_api.rs` is a small behavior-preserving owner-path cleanup whose old-baseline result is isolated, unlike the lower-count root `Config` candidate whose whole-file diagnostics churn identities.

## What Changes

- Replace the private `ExternAbi` import with the explicit `rustc_abi::ExternAbi::Rust` owner path at its single use site.
- Require the accepted 1,819-identity baseline to report exactly stale ID `b3:672b1993827f233b`, zero new IDs, and warning-only exit `0` before refresh.
- Regenerate the baseline as an exact removals-only set, lower canonical counts to 2,668 warnings and 1,818 identities, and preserve the permanent positive/negative ratchet semantics.
- Record bounded portfolio, regression, traceability, source-identity, and compatible-system evidence before deterministic archive.

## Impact

- **Files**: `src/safety/public_unsafe_api.rs`, the self-check baseline and documentation, Nix assertions, Cairn policy/traceability sources, and source-derived fixtures only if the repository-owned freshness rail requires them.
- **Testing**: focused positive and negative public-unsafe-API tests, old/refreshed canonical runs, exact baseline-set comparison, permanent self-check and traceability gates, policy freshness, object-corpus determinism, formatting, Clippy, Nextest, and full compatible-system flake validation.
