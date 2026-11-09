## Why

The genuine HWR deny-all run reports `max_imports` and `max_exports` as quantities because `numeric_units` finds `port` inside unrelated words. Adding a count suffix does not fix the classifier. Runtime APIs and wire schemas must not change to work around a substring false positive.

## What Changes

- Match quantity words as complete underscore-delimited Rust identifier components, retaining case normalization and existing unit/type rules.
- Exercise real compiler input containing import/export counters and an actual network-port quantity alongside the existing time/type fixtures.
- Replay the actual owning library and unchanged downstream deny-all policy; retain source-bound observations and publication non-claims.

## Impact

- **Files**: `src/naming/numeric_units.rs`, the maintained numeric UI fixture/snapshot, lint reference, changelog, and this native change.
- **Testing**: failing-before compiler evidence, owning library/unit/full UI checks, and the genuine HWR deny-all replay.
- **Owner/consumer**: Octet maintains the lexical classifier; HWR is the current consumer. No severity change, new exemption, runtime API migration, source substitution, or publication authority is introduced.
