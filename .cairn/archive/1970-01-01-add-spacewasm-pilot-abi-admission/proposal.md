# Proposal: Add exact SpaceWasm pilot ABI admission

## Why

The archived SpaceWasm MVP rail establishes bounded static facts for a hostless
core-module shape, but matching an export name and function kind does not establish
its resolved ABI. Molten's actual `molten.wasm.abi.v1` actor uses memory,
allocator/deallocator exports, packed results, and declared hostcalls. The separate
observe/pause diagnostic fixture must not stand in for that actor contract.

## What Changes

- Resolve imported/exported core function signatures through function/type indices
  and bind those signatures into canonical facts and interface identity.
- Require the existing hostless `run` export to have signature `()->()`.
- Add `spacewasm-molten-abi-pilot` for the real actor ABI and a separate
  `spacewasm-mvp-host-pilot` for the observe/pause diagnostic ABI.
- Bound input reads and reject oversized modules before parsing; check the selected
  feature set against the reviewed runtime support projection.
- Provide positive/negative fixtures and an operator smoke invoking actual pinned
  producer executables, artifact verification, and exact-byte tamper denial.

## Impact

Changes affect `cargo-octet/src/{wasm_artifact.rs,spacewasm.rs,evidence_rails.rs,lib.rs}`,
Nickel profile source and generated runtime input, existing operator documentation,
fixtures, and the changelog. The persisted receipt remains
`octet-evidence-rail-receipt/v1`, with static facts in `wasm_artifact`.

Mantle owns final module materialization and actual compiler/toolchain facts.
ChaosControl owns runtime observations; Molten owns exact per-manifest hostcall
admission and actual static-receipt intake. Parent integration owns publication
and immutable downstream producer pins. This change does not edit those providers.

## Non-goals

No runtime embedding, alternate engine, semantic/hardware/safety/release proof,
production flag change, or invented SpaceWasm release version. NASA source remains
`https://github.com/nasa/spacewasm` at
`e24cf09355a90497148eb5029fdb8e3400bd63e3`. Historical archived governance is unchanged.
