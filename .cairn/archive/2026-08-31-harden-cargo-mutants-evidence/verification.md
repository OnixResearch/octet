# Verification

Date: 2026-08-31

## Implemented boundary

- The typed cargo-mutants profile has BLAKE3 identity `b3:a40fc0726aea38932f42d6096a84846778e832472767b3c3129de4f7e69d9176`.
- Nix pins cargo-mutants 27.0.0 and includes it in the wrapped Octet runtime.
- The profile fixes package scope, arguments, budgets, retention, outcome policy, and non-claims.
- Bounded Exec owns process deadlines, output limits, and process-group teardown.
- The shell clears the child environment and rejects arbitrary commands.
- The core reads native `outcomes.json` and `mutants.json`. It does not parse terminal text.
- A successful unmutated baseline is required before a passing gate decision.
- Every inventory mutant must have exactly one terminal outcome.
- Caught, missed, timeout, unviable, not-run, and tool-failure outcomes remain separate.
- Source-manifest identities bind each mutant source path.
- Raw outcomes, inventory, logs, diffs, and the normalized projection retain BLAKE3 identities.
- Existing normalized imports remain lower-authority transcript evidence without the hardened profile.

## Evidence

- `nix develop --builders '' -c cargo test -p cargo-octet --lib mutation::` passed 4 tests.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib profiled_mutation` passed 1 integration test.
- The focused evidence-contract fixtures passed.
- Library and test Clippy gates passed with all features and `-D warnings`.
- Nickel typecheck, export freshness, policy registry, and Cairn traceability checks passed.
- The new mutation modules report zero Octet findings.
- A live cargo-mutants probe produced `outcomes.json`, `mutants.json`, logs, and retained diffs.
- Cairn validation and proposal, design, and tasks gates passed with the canonical workspace policy.

## Inherited blockers

The broad cargo-octet suite retains two pre-existing Nickel identity failures. The broad self-check retains its pre-existing warning-budget drift.

This change does not replace a baseline or suppress a finding.

## Non-claims

Mutation outcomes sample test-suite sensitivity. They do not prove test completeness, source correctness, whole-program correctness, or release eligibility.
