# Proposal: Exact Molten component effects artifact profile

## Why

The published checker prerequisite at `0e0e5dc8461b8b32c286de362c290b3f3ea2deee`
provides the already archived `molten-component-baseline`, including narrow bulk
memory and padded indirect-call encoding admissions. Its lifecycle is retained
in `.cairn/archive/1970-01-01-add-molten-component-artifact-profile/`; this change
does not reopen or rewrite it.

The genuine normalized effect guest exposes separate static extraction defects:
metadata names flattened across nested binaries, internally supplied core imports
treated as ambient authority, and a bound table import counted as an independent
third table. A function name/type-index allowlist cannot safely admit the exact
product function signature. Fix extraction and introduce a separate closed effect
profile, not broader baseline policy or arbitrary adapter allowances.

## What Changes

- Scope custom metadata to its actual nested binary.
- Separate validated locally bound imports from unresolved external authority,
  retaining internal import evidence and conservative unknown-resource denials.
- Count independent definitions and local/enclosing instantiation multiplicities;
  do not count resource aliases as new allocations.
- Add `molten-component-effects-v1`: exact canonical byte-list type bookkeeping,
  effect-request import and invoke export; no WASI or extra product functions;
  two independent tables and otherwise baseline static resource limits.
- Reuse the published granular `call-indirect-overlong` feature for the real
  LLVM-padded table-zero encoding, never enable general reference types or Wasm2.
- Retain exact-byte/profile/registry/cohort identity separation and recorded-only
  non-claims, using a v2 interface digest domain for corrected boundary facts.
- Add paired behavioral tests, an admitted static seed and native evidence.

## Impact

`cargo-octet/src/wasm_artifact.rs`, the Nickel profile source/contract and generated
projection, the artifact guide, the source-owned positive WAT seed, and this new
native Cairn lifecycle package. Both published baseline profile DTOs/policies and
their existing regression coverage remain intact.

## Non-goals

No authentication of a package/world name from metadata. No execution,
canonical Preserves, effect interpretation, capability, semantic-equivalence,
provenance, runtime-capacity or release claim. Mantle and product consumers retain
their respective build, operational and authority decisions. The static seed is
not an actor implementation. Publication, downstream immutable pins and lifecycle
archive remain parent-owned after integrated verification and gates.
