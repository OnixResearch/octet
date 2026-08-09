## Why

Octet already adopts Hegel RS through an archived Cairn change, an optional evidence rail, and one in-tree property law.

The current `--run` boundary still accepts an arbitrary explicit command and parses combined process output as a Hegel summary. This weakens provider identity and consumer usability.

## What Changes

- Replace arbitrary Hegel run commands with typed Nickel producer profiles and fixed argv construction.
- Require a dedicated versioned summary artifact instead of parsing mixed test-harness output.
- Remeasure package, lockfile, source, generator, profile, and produced-artifact identities before receipt publication.
- Add a selected Hegel law catalog for pure Octet cores, with positive and negative generated cases.
- Preserve compatible import-only use for externally produced summaries.

## Impact

- **Specs**: `dynamic-evidence-rails`, `law-tested-deterministic-cores`
- **Files**: Hegel profile Nickel, generated policy, producer adapter contract, run shell, receipt DTOs, law catalog, fixtures, artifact validation, and consumer documentation
- **Testing**: profiled runs, imports, property failures, malformed summaries, command mismatch, stale identities, unsafe paths, limits, and default-path independence

## Non-goals

- Do not make Hegel part of starter `cargo octet check`.
- Do not use Hegel for shell effects, UI output, Dylint diagnostics, or formal proof.
- Do not replace cargo-fuzz, Kani, Miri, Loom, concrete fixtures, or regression tests.
- Do not grant arbitrary executable, argument, path, environment, or network authority.

## Completion Evidence

The change is complete when profiled and imported Hegel evidence verifies deterministically, arbitrary commands fail closed, and selected laws produce replayable failures.
