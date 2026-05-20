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

The current bounded pilot command is:

```bash
cargo octet object supplement emit --family hir --function checked src/lib.rs --output-format json
cargo octet object supplement check --family hir --receipt hir-supplement.json --function checked src/lib.rs --output-format json
```

The pilot HIR supplement is source-token bounded. It reports contract status
`pilot-supported` only for the checked pilot emit/check surface and remains an
attached supplement to `octet.function-object.v1`.

The current rustc-backed HIR spike is narrower than general HIR support and is
selected-fixture-backed/toolchain-scoped. Omit `--fixture-shape` to use the
default checked arithmetic fixture, or select one of the closed fixture shapes:

```bash
cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --output-format json
cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --receipt hir-spike.json --output-format json
cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape branch-control-flow --output-format json
cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape branch-control-flow --receipt hir-spike.json --output-format json
```

The rustc HIR fixture spike records `octet-rustc-hir-fixture-spike` extractor
identity, exact `rustc -vV`/host identity, selected fixture shape
(`checked-arithmetic` or `branch-control-flow`), fixture path/function context,
HIR observation hash, extraction caveats, and fail-closed replay comparisons. It
is not general HIR support; arbitrary-path and non-fixture HIR remain
planned/unsupported until their own positive and negative deterministic evidence
exists.

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

The current bounded pilot command is:

```bash
cargo octet object supplement emit --family mir --function checked src/lib.rs --output-format json
cargo octet object supplement check --family mir --receipt mir-supplement.json --function checked src/lib.rs --output-format json
```

The pilot MIR supplement records explicit unknown/unsupported observations for
summary families that the source-token pilot cannot prove. It reports contract
status `pilot-supported` only for the checked pilot emit/check surface, while
THIR and any unimplemented non-pilot HIR/MIR family remain `unsupported-planned`.

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
