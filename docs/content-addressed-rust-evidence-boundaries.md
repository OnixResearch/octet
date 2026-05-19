# Content-addressed Rust evidence boundaries

This document fixes the Phase 7 boundary for Octet's Valence-oriented content-addressed Rust evidence work. It is a contract document, not an implementation claim for HIR/MIR extraction.

## Canonical v1 identity

`octet.function-object.v1` remains the canonical v1 review/provenance identity for selected Rust functions.

- The v1 object hash is a BLAKE3 identity over canonical source-token object content.
- The v1 object hash is evidence identity for review, replay, diffs, bundles, and name pointers.
- The v1 object hash is not a proof of behavioral correctness, semantic equivalence, build correctness, or distributed execution correctness.
- HIR, THIR, and MIR records MAY supplement a v1 object, but they do not replace the v1 object hash as the v1 review identity.

## HIR supplemental evidence boundary

Future HIR-derived evidence is toolchain scoped.

A future HIR evidence record MUST include:

- the referenced `octet.function-object.v1` object hash;
- exact rustc/toolchain identity;
- extractor name and version;
- HIR evidence schema version;
- resolved dependency summary where available;
- unresolved dependency, macro-origin, generated-code, and cfg caveats where applicable.

HIR evidence MAY improve dependency summaries and macro-origin reporting. It MUST NOT claim cross-toolchain stability unless a separate verifier proves that claim for the named toolchains and corpus.

## MIR supplemental evidence boundary

Future MIR-derived evidence is compiler-version-coupled.

A future MIR evidence record MUST include:

- the referenced `octet.function-object.v1` object hash;
- exact rustc/toolchain identity;
- extractor name and version;
- MIR evidence schema version;
- summaries for call, panic, drop, async, unsafe, allocation, and effect surfaces only when the extractor can support them;
- explicit unknown/unsupported caveats for every summary family that is absent or incomplete.

MIR evidence MAY support stronger review, cache, or effect decisions. It MUST NOT claim stable identity across compiler versions, target triples, optimization modes, feature flags, or dependency graphs unless those dimensions are explicitly part of the compared evidence and a separate verifier proves equivalence.

## Downstream ownership boundary

Octet owns Rust extraction and object emission.

- Octet owns source scanning, function-object records, object hashes, object markers, name-pointer manifests, object diffs, caveat summaries, bundles, and replay metadata emitted by `cargo octet`.
- Trellis owns verified abstract admission models for definitions, dependency closures, names, recursive groups, and ability/effect admission. Trellis does not own Rust source extraction.
- Cairn owns lifecycle policy, gate execution, replay receipts, and receipt validation around Octet bundles. Cairn consumes Octet receipts without owning or redefining Rust extraction semantics.
- Crunch/Mantle may consume Octet object and bundle hashes as build/proof evidence identities. Octet hashes alone do not prove build correctness or binary reproducibility.
- Aspen may consume Octet object and bundle hashes as CI, forge, and runtime evidence identities. Octet hashes alone do not prove distributed execution correctness or live-system safety.

## Pilot boundary

Crunch/Mantle and Aspen pilots are bounded consumers of Octet evidence.

A pilot contract SHOULD name:

- selected source paths or crate profiles;
- Octet command and version;
- object/bundle schema versions;
- checker/config/source/executor hashes used by the consumer;
- caveats that block pure-cache or stronger admission;
- replay command and receipt path.

A pilot contract MUST keep correctness claims outside Octet object hashes unless a downstream verifier supplies separate proof evidence.

## Required negative language

When describing this layer, use bounded wording:

- accepted: "content-addressed review/provenance identity";
- accepted: "bounded local determinism receipt";
- accepted: "toolchain-scoped HIR/MIR supplemental evidence";
- rejected: "function hash proves correctness";
- rejected: "MIR hash is stable across rustc versions";
- rejected: "Octet owns Cairn lifecycle policy";
- rejected: "Octet hashes prove Aspen/Crunch execution correctness".
