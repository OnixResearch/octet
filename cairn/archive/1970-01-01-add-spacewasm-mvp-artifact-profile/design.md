## Context

The general Octet Wasm artifact rail is designed around one pinned wasm-tools cohort, execution-free parsing, exact artifact identity, structured findings, and bounded Valence exports. SpaceWasm currently supports a strict core-module subset and has no stable release. Its README support matrix and source implementation can change, while module producer defaults may enable unsupported proposals.

The correct Octet role is a profile over statically observable bytes, not a wrapper around the SpaceWasm runtime. Runtime differential behavior belongs in ChaosControl, build/materialization facts in Mantle, and runtime admission in Molten or another consumer.

## Decisions

### 1. The profile extends the general wasm-tools analysis core

`spacewasm-mvp` will reuse the admitted wasm-tools parser/fact model and report bundle. Octet will not import SpaceWasm decoder/interpreter code into the static rail and will not execute host tools as part of artifact decisions.

### 2. One exact SpaceWasm cohort defines profile intent

A typed Nickel profile will bind the reviewed SpaceWasm source/materialization identity, support-matrix projection, core binary kind, allowed proposals, import/export ABI, memory/table/start/growth rules, structural/resource ceilings, severity, and non-claims. An upstream update creates a new profile identity and requires full corpus replay.

### 3. Support-matrix facts are checked projections

The human-authored profile is authoritative for Octet policy. A generated projection will record upstream observed support and the review source/commit. Drift between the reviewed source and generated runtime input fails profile freshness rather than silently widening accepted features.

### 4. Static facts and runtime likelihood are not confused

A pass means the bytes satisfy all checked static constraints. It does not assert that SpaceWasm has no decoder bug, enough configured allocator pages, compatible host implementations, deterministic floating-point behavior, or successful execution. Findings and summaries must state that boundary.

### 5. Resource posture is explicit

The profile will classify declared linear-memory minimum/maximum, tables, function parameters/locals, structural counts, start behavior, and `memory.grow`. Limits derived from a SpaceWasm cohort are named profile fields rather than scattered literals. Interpreter allocator/code/stack configuration that is not encoded in the artifact remains external runtime configuration.

### 6. Identity domains stay separate

Exact module bytes receive an artifact BLAKE3. The SpaceWasm source/materialization cohort and policy profile receive distinct identities. An optional normalized ABI/interface identity cannot replace exact bytes or imply behavioral equivalence.

### 7. External runtime reports remain recorded-only

Verified Mantle, ChaosControl, or Molten report bundles may be linked after their own identity verification. Octet records producer, cohort, artifact, result, role, and non-claims but does not infer static truth from runtime success or runtime correctness from static success.

### 8. Findings are deterministic and actionable

The profile will distinguish wrong binary kind, unsupported proposal, forbidden import/WASI, ABI mismatch, memory/table/growth violation, structural limit, malformed metadata, stale cohort/profile, external-evidence mismatch, and overclaim classes in stable order.

## Functional core / imperative shell split

- **Pure core**: profile/support-projection validation, byte fact analysis through the shared parser core, policy evaluation, identity material, finding ordering, external-evidence DTO validation, and report construction.
- **Imperative shell**: profile export reads, artifact reads, bundle reads/writes, optional verified external-report loading, fixture discovery, and human/JSON/SARIF rendering.

## Risks / Trade-offs

- A restrictive profile may reject normal Rust `wasm32-unknown-unknown` output. That is an actionable producer mismatch, not a reason to widen silently.
- Upstream support may land faster than Octet profile updates. Exact cohort pinning intentionally preserves review lag.
- Static operator classification may not capture every engine implementation constraint. Runtime compatibility remains separately tested.
- SpaceWasm requirements documents may lag source. The profile update process must inspect code/tests/issues and record discrepancies rather than trusting one prose file.

## Non-Goals

- No SpaceWasm execution, embedding, benchmarking, or fuzz-runtime orchestration in Octet.
- No Component Model, WIT, or WASI compatibility claim for SpaceWasm.
- No automatic lowering or rewriting of rejected modules.
- No claim that profile conformance proves runtime success, memory safety, certification, or release eligibility.
- No replacement for the general Wasm component artifact profile.
