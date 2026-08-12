## Phase 1: Schema and collector boundary

- [x] [serial] Define Rust-owned architecture shard, normalized IR, coverage, issue, and receipt DTOs with schema versions r[octet.compiler_architecture_ir.schema]
- [x] [serial] Generate Nickel contracts and register the Rust-derived Architecture IR receipt family r[octet.compiler_architecture_ir.schema]
- [x] [serial] Add collector activation only for a valid active project architecture policy r[octet.compiler_architecture_ir.activation]
- [x] [serial] Add bounded per-compilation-unit shard emission through the rustc driver adapter r[octet.compiler_architecture_ir.facts]

## Phase 2: Compiler facts and identities

- [x] [serial] Collect package, target, feature, role, module, item, type, trait, implementation, signature, call, constructor, effect, expansion, and configuration facts r[octet.compiler_architecture_ir.facts]
- [x] [serial] Record trait dispatch, dynamic dispatch, FFI, macro, generated, and unsupported facts as typed unknowns r[octet.compiler_architecture_ir.unknowns]
- [x] [serial] Add canonical BLAKE3 identities without using rustc local indexes as durable IDs r[octet.compiler_architecture_ir.identity]
- [x] [serial] Add named policy limits for units, facts, unknowns, spans, shards, and final artifacts r[octet.compiler_architecture_ir.boundary]

## Phase 3: Pure normalization and coverage

- [x] [serial] Implement pure shard schema, identity, bound, reference, order, and deduplication validation r[octet.compiler_architecture_ir.schema]
- [x] [serial] Derive expected compilation units from locked Cargo metadata, declared targets, policy scope, and exact invocation r[octet.compiler_architecture_ir.coverage]
- [x] [serial] Compare expected units with valid shards and reject missing, duplicate, stale, malformed, or mismatched coverage r[octet.compiler_architecture_ir.coverage]
- [x] [serial] Emit canonical Architecture IR and coverage artifacts with explicit non-claims r[octet.compiler_architecture_ir.boundary]

## Phase 4: Verification

- [x] [parallel] Add positive fixtures for resolved traits, implementations, aliases, signatures, constructors, calls, expansion mapping, and active configuration r[octet.compiler_architecture_ir.verification]
- [x] [parallel] Add negative fixtures for comments, inactive code, dynamic dispatch, indirect factories, missing units, stale shards, malformed facts, and limit overflow r[octet.compiler_architecture_ir.verification]
- [x] [parallel] Repeat collection and normalization under unchanged inputs and compare byte identities r[octet.compiler_architecture_ir.identity]
- [x] [serial] Prove projects without an architecture policy do not invoke collection or emit new artifacts r[octet.compiler_architecture_ir.activation]
- [x] [serial] Run focused collector tests, contract freshness, artifact validation, formatting, Cairn validation, gates, and traceability coverage r[octet.compiler_architecture_ir.verification]