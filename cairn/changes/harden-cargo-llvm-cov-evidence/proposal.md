## Why

Octet already exposes an optional `coverage-baseline` rail with fixed cargo-llvm-cov argv and a normalized coverage summary contract.

The rail hashes a baseline file but trusts precomputed regression fields. It does not compare native current and baseline counts under a typed scope and toolchain policy.

## What Changes

- Add typed Nickel coverage profiles for tool identity, scope, features, metrics, thresholds, comparison policy, limits, and non-claims.
- Consume pinned cargo-llvm-cov machine output through a dedicated artifact path.
- Normalize covered and total counts before deriving percentages under explicit precision rules.
- Compare current and accepted baseline artifacts inside a pure core with strict comparability checks.
- Preserve compatible normalized imports as lower-authority transcript evidence.

## Impact

- **Specs**: new `coverage-baseline-evidence` capability
- **Files**: coverage profile Nickel, pinned provider package, native-output adapter, comparison core, receipt DTOs, fixtures, artifact validation, and consumer documentation
- **Testing**: stable coverage, regressions, threshold failures, incomparable baselines, malformed reports, stale identities, path normalization, limits, and default-path independence

## Non-goals

- Do not make coverage collection part of starter `cargo octet check`.
- Do not claim that executed lines, regions, functions, or branches are behaviorally correct.
- Do not compare reports from different scopes, targets, features, toolchains, or profile policies.
- Do not grant arbitrary executable, argument, path, environment, or network authority.

## Completion Evidence

The change is complete when current and baseline counts compare deterministically, incomparable inputs fail closed, and native artifacts remain independently verifiable.
