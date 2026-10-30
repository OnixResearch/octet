# Design: Exact SpaceWasm pilot ABI admission

## Decisions

### Resolve ABI independently of export names

`spacewasm.rs` is a deterministic parser/fact core. It maps imported and defined
function indices through the core type section, then emits sorted function ABI
facts containing direction, namespace, name, parameters, and results. Interface
identity includes these facts. Exact-byte artifact identity remains a separate
BLAKE3 domain. Existing component analysis remains on its existing path.

### Separate actors from diagnostics

`spacewasm-molten-abi-pilot` requires exported memory, `molten_alloc(i32)->i32`,
`molten_dealloc(i32,i32)->()`, and `molten_run(i32,i32)->i64`. The closed optional
import set is `molten:hostcall` assert/send, each `(i32,i32)->i64`; import type-section
indices are not ABI identity. The consumer must require the exact manifest subset.
The memory must be unshared wasm32, have at least one initial page and an explicit
maximum no greater than 256 pages. Tables, start, and growth are denied.

`spacewasm-mvp-host-pilot` independently admits observe(i32), pause(), run(), and
an exported mutable i32 total. That diagnostic contract cannot admit an actor.
The archived hostless profile remains separate and its run signature is tightened.

### Keep the reviewed intersection and immutable provenance

Profiles remain Nickel-authored and deterministically exported. The selected
proposal set must be contained in reviewed support and disjoint from denied
proposals. The granular mutable-globals token enables that feature alone.
The NASA revision, source lock, support projection, and runtime cohort remain
unchanged; Wasmtime comparison versions are runtime-provider facts, not static
SpaceWasm artifact facts. Producer custom sections are not authenticated provenance.

### Bound the effect shell and preserve original-byte identity

The CLI shell reads at most the selected byte ceiling plus one byte and rejects
oversized inputs without emitting a truncated-byte identity. The pure analyzer
also checks the ceiling before validator/parser construction. Exact identities are
computed only over the original bytes supplied to the analyzer.

### Consume verified persisted receipts

CLI output is `evidence-rail-receipt.json`, schema
`octet-evidence-rail-receipt/v1`, with `wasm_artifact` details; the internal analysis
envelope is not a substitute persisted contract. Consumers run the pinned actual
producer's artifact verifier, require passed status and no error findings, compare
candidate bytes to bundled `wasm-artifact.bin`, and bind selected profile and NASA
source/cohort. A self-hashed external assertion is not static proof.

## Concurrency and ownership

Profile generation follows ABI/profile implementation. Fixtures and operator smoke
can be authored independently after the ABI is fixed. Final tests, actual producer
smoke, publication, and lifecycle gates are parent-owned after the provider snapshot
is frozen. No historical archive or root Molten Cargo/Nix wiring is modified here.

## Verification and boundary

Parent runs focused existing and added Wasm/SpaceWasm tests, profile freshness,
and `python3 standards/wasm-artifact/spacewasm/smoke.py /absolute/cargo-octet /absolute/wasm-tools`
with actual pinned executables. The smoke must observe valid actor admission and
successful artifact verification, then exact-byte tamper denial; unbounded memory,
wrong allocator/host, WASI, component, and unsupported proposal cases must deny.
Runtime pointer/descriptor behavior, allocator correctness, fuel, stack, host
semantics, continuation, hardware, safety, and release eligibility remain nonclaims.
