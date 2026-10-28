# Proposal: Add a Molten component artifact profile

## Why

Molten's supported component world is denied by `portable-component-baseline`:
two nested `component::preserves-payload::Type(Eq(0))` imports and bulk memory.
The baseline must stay import-free, so the fix is a separate reviewed profile
plus a granular proposal token.

## What Changes

- Add `molten-component-baseline`: component kind, the declared preserves-payload
  type import only, bulk memory, no ambient imports, baseline resource limits.
- Recognize a granular `bulk-memory` proposal mapped to wasmparser's bulk-memory
  feature instead of admitting all of Wasm 2.
- Keep `portable-component-baseline` unchanged.

## Impact

- `standards/wasm-artifact/profiles.ncl` and its generated projection.
- `cargo-octet` proposal vocabulary, feature mapping, and tests.

## Non-goals

- No weakening of the shared baseline.
- No admission of other imports, proposals, or Molten runtime authority.
