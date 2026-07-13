# SpaceWasm MVP artifact profile

`spacewasm-mvp` is an execution-free Octet profile over exact WebAssembly core-module bytes. It is a restrictive, hostless `run`-export ABI review profile; it is not a universal SpaceWasm ABI and does not execute or embed SpaceWasm.

## Reviewed cohort

- Repository: `https://github.com/nasa/spacewasm`
- Git revision: `e24cf09355a90497148eb5029fdb8e3400bd63e3`
- Derived cohort identity: `b3:7c537eef7bff66d0a90508715de08de79c9aaac23352e1007a6d47e7fb4674a2`
- Upstream `Cargo.lock` identity: `b3:4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1`
- Reviewed sources: `Cargo.lock`, `README.md`, `REQUIREMENTS.md`, `src/SPEC.md`, `src/code.rs`, `src/compiler.rs`, `src/opcode.rs`, and `src/types.rs`

The upstream Git revision is a Git SHA-1 interoperability identifier. Octet derives profile, projection, artifact, interface, report, and cohort identities with BLAKE3.

`support-projection.ncl` records the reviewed support boundary. Its checked JSON export is embedded in the runtime profile. The review found that `REQUIREMENTS.md` permits selected post-MVP proposals in principle while the README matrix and reviewed decoder expose WebAssembly 1.0 plus mutable globals. Octet therefore admits only wasmparser's `WASM1` feature set and fails closed on the broader proposals.

## Static profile

The profile requires:

- a core module, never a component, WIT, or WASI artifact;
- no imports;
- exactly allowed exports, with `module::run` required;
- no start function and no `memory.grow`;
- at most one memory and one table;
- named limits for artifact size, functions, pages, table elements, parameter words, and local words;
- recorded-only producer metadata and deterministic duplicate-section rejection.

The profile limits are intentionally no wider than the reviewed upstream support limits. Runtime allocator pages, code pages, control frames, stack depth, fuel, and host implementations remain external configuration and cannot be inferred from module declarations.

## Standalone and Mantle handoff

```console
cargo octet evidence collect \
  --rail wasm-artifact \
  --input module.wasm \
  --config standards/wasm-artifact/generated/profiles.json \
  --profile spacewasm-mvp \
  --artifact-dir artifacts/spacewasm
cargo octet artifact verify --artifact-dir artifacts/spacewasm
```

Mantle may supply exact final bytes and a `mantle:` parent artifact. Octet independently measures the bytes and records the handoff without importing store-path, build-success, materialization, sandbox, or release semantics.

## Optional runtime observations

Identity-verified Mantle, ChaosControl, or Molten observations can be linked with repeated `--external-report PATH`. Reports must bind the exact artifact, selected profile, reviewed SpaceWasm cohort, canonical report identity, recorded-only role, result, and no-claim-transfer boundary. Runtime success never changes a static denial.

## Corpora

`corpus.ncl` binds exact generated Wasm and external-report bytes, tool versions, seeds or seed artifacts, input identities, transforms, expected issue classes, and shrink predicates. The fast lane covers admitted and denied boundaries. The deep lane records deterministic wasm-smith and wasm-mutate outputs; these remain validation evidence, not proof.

## Non-claims

A pass does not prove SpaceWasm decode, instantiation, execution, memory safety, semantic equivalence, sandbox effectiveness, configured runtime capacity, flight qualification, production readiness, certification, or release eligibility.
