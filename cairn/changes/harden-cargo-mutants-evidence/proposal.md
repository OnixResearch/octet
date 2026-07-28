## Why

Octet already exposes an optional `mutation-testing` rail with fixed `cargo mutants --json` argv and bounded non-claims.

The rail accepts a thin normalized summary and does not bind a typed run profile, tool package, baseline test, mutation inventory, or native artifact set.

## What Changes

- Add typed Nickel profiles for cargo-mutants scope, tool identity, command shape, budgets, outcome policy, and non-claims.
- Consume the pinned provider's declared machine-readable artifacts instead of mixed human logs.
- Require a passing unmutated baseline before mutant outcomes can support a passing receipt.
- Bind the mutation inventory, each terminal outcome, source identity, toolchain, lockfile, profile, logs, and retained diffs.
- Preserve legacy imports as lower-authority transcript evidence without invented run measurements.

## Impact

- **Specs**: new `mutation-testing-evidence` capability
- **Files**: mutation profile Nickel, pinned provider package, parser core, bounded run shell, receipt DTOs, fixtures, artifact validation, and consumer documentation
- **Testing**: clean baseline, caught and survived mutants, timeouts, unviable mutants, malformed output, stale identity, incomplete runs, limits, and default-path independence

## Non-goals

- Do not make mutation testing part of starter `cargo octet check`.
- Do not treat a mutation score as proof of test completeness or source correctness.
- Do not hide survived, timed-out, unviable, skipped, or tool-failure outcomes inside aggregate counts.
- Do not grant arbitrary executable, argument, path, environment, or network authority.

## Completion Evidence

The change is complete when profiled runs and compatible imports verify deterministically, incomplete outcome sets fail closed, and every surviving mutant remains reviewable.
