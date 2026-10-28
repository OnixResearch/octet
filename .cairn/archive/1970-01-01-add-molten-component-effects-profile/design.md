# Design: Exact Molten component effects artifact profile

## Reviewed scope and boundary semantics

Use parser Version/End nesting to assign deterministic binary scope ordinals.
The duplicate metadata policy compares `(scope, section-name)`, never names
flattened across nested modules/components. Exact bytes remain unchanged.

Validation must succeed before a local instantiation proves imports internally
bound. Track local core-module/component index spaces and instantiation counts;
retain imports from unused or exported reusable definitions. Unresolved nested
component imports use scope-qualified namespaces so root allowances cannot
authorize them accidentally. Internal imports are still recorded in the receipt
as `bound_imports`, with their source scope. Only root exports form the external
surface. An unresolved generative alias cannot erase ambient imports; imported
or aliased instantiations without locally known implementations fail resource
accounting closed.

Count definitions per local instantiation, multiplying enclosing component
instances with saturating arithmetic; conservatively count unused definitions
once. Imported aliases do not allocate independent resources when validation
proves their supplying instance. Retain all definition declaration bounds and
record imported constraints. The observed guest defines tables 5/5 and 1/1,
with one bound import alias to the latter: two independent tables, not three.
Repeated instantiations still count repeatedly. Static absent maxima remain
absent and are not represented as runtime bounds or capacity evidence.

## Exact effect profile

Optional `component_interface = "structural-v1"` selects resolution through
validated component types, independent of local type indices or debug output.
`required_imports` is an optional additive profile field. Empty/absent additions
are omitted by Rust serialization to preserve existing profile DTO identities.
The effects profile requires these exact root items:

- `component::preserves-payload::Type(list<u8>)` (type equality bookkeeping).
- `component::effect-request::Func(list<u8>)->result<list<u8>,string>`.
- Sole export `component::invoke::Func(list<u8>)->result<list<u8>,string>`.

It permits wasm1, component-model, bulk-memory and call-indirect-overlong only;
max imports 2, exports 1,
memories 1, independent tables 2; other baseline resource limits and denial
postures remain unchanged. Unsupported structural kinds do not match. No WASI,
extra product functions, wrong signatures, or missing required boundary items
are admitted. The supported `molten:component-effects@1.0.0` / `actor-effects`
world name is product-owned context, not an authenticated checker fact.

The padded table-zero call-indirect immediate at offset `0x7f5` of the retained
real normalized guest requires only wasmparser's `CALL_INDIRECT_OVERLONG` flag.
Reuse this already published granular token; do not enable general reference
types, SIMD, or all Wasm2. Paired tests admit canonical and overlong zero table
indices while denying an undeclared nonzero table index and reference operations.

The interface digest domain advances to `octet-wasm-interface/v2` because the
extracted external boundary changes. Exact bytes, canonical profile DTO, registry
bytes, cohort identity, and recorded-only receipts remain distinct. No matching
shape or metadata transfers canonical Preserves, semantic, capability, build,
runtime, provenance, or release authority to Octet.

## Verification handoff

Paired tests cover nested metadata versus same-scope duplicates; bound adapters
versus unused/exported imports; repeated instantiation versus import aliasing;
canonical type reindexing versus wrong function/type surface; extra WASI/product
imports; and independent table/count/element boundaries. Source-owned static WAT
is only an identity seed, not fake actor execution. Focused worker verification
is delegated by the parent after source integration; the parent retains whole
cohort execution, lifecycle gates, publication, and archive authority. Preserve
initial compiler/real-guest denials alongside final exact-byte collection and
artifact verification evidence; no earlier passing stale-base run substitutes
for the clean published-baseline replay.

