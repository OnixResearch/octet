## Context

Octet standards architecture checks currently scan source strings. Dylint checks use HIR and type information, but they emit independent diagnostics rather than a reusable project graph.

A strong architecture gate needs exact compiler observations and explicit records for facts that the collector cannot resolve.

## Decisions

### Decision: Collect facts only for active project policies

**Choice:** `cargo-octet` starts architecture collection only after successful project-policy activation.

Unconfigured projects do not receive collector environment, collector work directories, Architecture IR artifacts, or added compiler work.

**Rationale:** Compiler architecture analysis follows the project opt-in boundary.

### Decision: Emit one shard per compiled unit

**Choice:** The rustc driver adapter emits one bounded fact shard for each selected crate target and compilation configuration.

Each shard name uses a BLAKE3 identity over the package, target, crate type, features, toolchain, policy, and source envelope.

The shell owns shard directories and file writes. The collector only derives facts from supplied compiler contexts.

**Rationale:** Cargo can compile units concurrently. Unique content identities prevent shared-file races.

### Decision: Define a Rust-owned Architecture IR

**Choice:** Versioned Rust DTOs own emitted fact and receipt schemas. Generated Nickel contracts cover review and compatibility.

The normalized IR records:

- policy, toolchain, target, package, feature, and source identities;
- expected and observed compilation units;
- project role assignments;
- modules, functions, types, traits, and implementations;
- public signature type references and crate origins;
- call sites and resolved local or external definitions;
- constructor sites and constructed type origins;
- direct effect primitives and capability families;
- macro expansion origins and user call sites;
- active conditional-compilation facts;
- trait, dynamic-dispatch, FFI, and other unresolved facts;
- bounded source spans and stable issue identities.

**Rationale:** Runtime outputs and compatibility belong to Rust-owned DTOs.

### Decision: Do not use rustc local IDs as durable identity

**Choice:** Local `DefId`, HIR, and MIR indexes remain transient collector data.

Durable item IDs use BLAKE3 framing over the schema domain, package identity, normalized qualified path, item kind, signature identity, and source envelope.

**Rationale:** Compiler-local indexes can change across builds and toolchains.

### Decision: Use compiler semantics for gate-grade facts

**Choice:** Trait definitions, implementations, type origins, signatures, calls, constructors, and expansion origins come from rustc HIR, type checking, and selected MIR facts.

Source spelling can remain display evidence. It cannot satisfy a gate-grade declaration or implementation obligation.

**Rationale:** Comments, aliases, formatting, and inactive source must not satisfy compiler-visible architecture rules.

### Decision: Preserve unknown facts

**Choice:** Ambiguous trait dispatch, unsupported macros, missing bodies, opaque generated code, unresolved definitions, and unsupported compiler shapes emit typed unknown records.

The normalizer never drops an unknown record. Later mode evaluation decides its severity.

**Rationale:** Unknown is evidence. Silence is not evidence.

### Decision: Prove collection coverage separately

**Choice:** The shell derives expected units from locked Cargo metadata, selected targets, project scopes, and the exact Cargo invocation.

A pure coverage evaluator compares expected units with valid shards. Missing, duplicate, malformed, stale, or mismatched shards fail collection.

**Rationale:** A clean subset cannot stand for the declared production universe.

### Decision: Keep normalization pure

**Choice:** A deterministic core validates shard schemas, identities, bounds, references, and ordering. It returns normalized IR or diagnostics.

The shell reads shard bytes, captures Cargo status, and writes final artifacts.

**Rationale:** Architecture meaning remains directly testable without Cargo or rustc processes.

### Decision: Bound every collection surface

**Choice:** Named policy values limit shards, units, items, types, traits, implementations, calls, effects, unknowns, span bytes, and artifact bytes.

Limit overflow returns a deterministic incomplete-collection diagnostic.

**Rationale:** A compiler analysis rail must not introduce unbounded evidence collection.

## Architecture IR Flow

```text
active project policy
  -> cargo metadata and selected targets
  -> rustc collector adapter per compilation unit
  -> bounded fact shards
  -> pure normalization and coverage evaluation
  -> canonical Architecture IR + coverage report
```

## Functional Core and Imperative Shell

The collector adapter reads rustc contexts and creates in-memory facts. The cargo-octet shell owns process orchestration and file effects.

The normalizer core accepts policy, expected units, and shard values. It returns canonical IR, coverage, or deterministic errors.

## Risks / Trade-offs

- rustc-private APIs can drift. Exact toolchain and collector identities plus the versioned schema make drift visible.
- MIR extraction can increase build cost. Opt-in activation and explicit limits contain the cost.
- Macro and trait resolution remain incomplete. Typed unknown facts preserve that boundary.
- Generated code can create large fact sets. Separate role scopes and limits prevent silent omission.

## Validation

Positive fixtures cover resolved traits, adapter implementations, public signatures, constructors, local calls, macro call-site mapping, active configuration, and repeat identity.

Negative fixtures cover comments that mimic traits, renamed imports, inactive implementations, indirect factories, dynamic dispatch, missing units, stale shards, duplicate IDs, malformed spans, limit overflow, and unsupported facts.