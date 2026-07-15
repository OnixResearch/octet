## Context

The external Clippy rail already preserves Clippy diagnostic identities and writes a verified static-lint receipt. Its run command is currently fixed to default `cargo clippy --message-format=json`, however, and `--config`/`--profile` do not select a reviewed restriction policy. The generic runner also concatenates stdout and stderr before JSON parsing, which can turn ordinary Cargo progress text into malformed-input diagnostics, while a nonzero tool exit without a Clippy-coded message is not itself fail-closed evidence.

Octet already establishes the relevant architecture elsewhere: Nickel owns human-authored profile policy, runtime Rust consumes checked-in generated JSON, pure cores validate/normalize policy, thin shells read files and spawn tools, and BLAKE3 identities bind reviewed configuration into receipts.

## Decisions

### 1. Add a separate Nickel-owned Clippy restriction catalog

**Choice:** Add `standards/clippy-restrictions/profiles.ncl`, a typed contract, generated JSON, and positive/negative fixtures. The catalog will use schema `octet-clippy-restriction-profiles/v1` and each profile will declare an ID, summary, `warn` or `deny` lint level, a non-empty explicit lint list, claim scope, and non-claims.

**Rationale:** Clippy restrictions are external compiler policy, not Octet Dylint identities. Keeping the catalog separate avoids pretending Clippy lints are Octet lints while still giving the stack one reviewable source of truth.

### 2. Ship narrow profiles, never the blanket group

**Choice:** Ship:

- `production-safety`: `dbg_macro`, `mem_forget`, `undocumented_unsafe_blocks`, and `multiple_unsafe_ops_per_block` at `deny`;
- `canonical-determinism`: the production set plus `host_endian_bytes` and `iter_over_hash_type` at `deny`;
- `shell-boundary-inventory`: `exit` at `warn` for gradual functional-core/imperative-shell cleanup.

Runtime validation will allow only the reviewed lint inventory and reject duplicate lint IDs, unknown lint IDs, duplicate profile IDs, empty fields, invalid claim scope, and the blanket `restriction` group.

**Rationale:** The selected lints close concrete gaps without duplicating Octet's `no_unwrap`, `no_panic`, `no_todo`, checked arithmetic, narrowing, and suppression-governance lints. Native-endian and unordered-iteration restrictions remain opt-in for canonical workspaces, while process-exit cleanup starts as inventory because shell-heavy repositories have a material backlog.

### 3. Keep command construction pure and execution fixed

**Choice:** A pure Rust module will parse and validate the catalog, select a profile, compute its BLAKE3 identity, and build exact argv:

```text
cargo clippy --quiet --workspace --all-targets --message-format=json -- <LEVEL> clippy::<LINT> ...
```

The evidence-rail shell will read the generated JSON, reject `--command` for profiled Clippy runs, spawn only the constructed argv, parse Clippy JSON from stdout, and retain the tool exit code. Unprofiled run mode retains the existing default command.

**Rationale:** This prevents profile files from becoming arbitrary process-execution authority and makes policy logic testable without Cargo, Clippy, or filesystem setup.

### 4. Bind policy and command into the existing receipt

**Choice:** Extend the evidence-rail receipt with optional config identity and Clippy restriction details: profile ID, selected-profile BLAKE3 identity, lint level, and exact lints. Profiled run receipts must carry a BLAKE3 config identity and exact reconstructed argv. Receipt validation will reject missing identities, profile/detail mismatches, command drift, unsupported lint IDs, and invalid BLAKE3 shapes.

**Rationale:** A profile label or config path alone does not prove which reviewed bytes or flags were used. The selected identity supports stable comparison while full config identity detects unrelated catalog drift.

### 5. Treat tool/process failures separately from lint findings

**Choice:** Clippy stdout remains the JSON diagnostic input. Nonzero tool exits add a deterministic receipt diagnostic, including cases where compilation fails without a Clippy-coded finding. Stderr is not parsed as JSON.

**Rationale:** Cargo progress and linker text are not Clippy JSON. Separating channels preserves parse precision while the exit-status diagnostic prevents false clean receipts.

### 6. Preserve evidence and authority boundaries

**Choice:** Profiled Clippy remains an opt-in `static_lint` evidence rail. It will not become part of starter `cargo octet check`, replace Octet source lints, or claim behavioral correctness, panic freedom, memory safety, deterministic runtime behavior, compiler correctness, formal proof, or release eligibility.

**Rationale:** Clippy provides valuable compiler diagnostics but does not transfer semantic or release authority.

## Data flow

```text
standards/clippy-restrictions/profiles.ncl
  -> Nickel typecheck + deterministic JSON export
  -> pure Rust catalog validation + profile selection
  -> fixed Clippy argv + selected-profile BLAKE3
  -> shell-owned cargo clippy execution
  -> stdout JSON findings + exit-status diagnostic
  -> config/profile/command-bound external-lint receipt
  -> cargo octet artifact verify
```

## Risks / Trade-offs

- Clippy lint availability follows the pinned Rust/Clippy toolchain; changing that pin requires replaying the profile fixtures and downstream inventories.
- `iter_over_hash_type` is broader than Octet's flow-aware `nondeterministic_output`; it therefore belongs only in the explicit canonical profile.
- `host_endian_bytes` can be correct at kernel/FFI boundaries; affected code needs narrow reasoned exceptions or a less strict profile.
- A deny profile returns a nonzero Cargo exit when it finds a violation. The receipt intentionally records both the Clippy finding and failed process status.
- The additive receipt fields preserve old imported receipts, but profiled run receipts are held to the stronger identity contract.
