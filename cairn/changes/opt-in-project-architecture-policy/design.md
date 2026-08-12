## Context

Octet currently stores first-party FCIS defaults in the global standards registry. The architecture checker applies those defaults when a project declares no crate-boundary mechanism.

That behavior is useful for Octet fixtures. It is not a safe consumer activation contract.

The existing port mechanism is opt-in, but crate-boundary and capability checks can still resolve fallback scopes. Strict Rust profiles also use `core_source_scopes` independently.

## Decisions

### Decision: Activate architecture only through project metadata

**Choice:** `[workspace.metadata.octet].architecture_policy` points to one workspace-relative Nickel file.

Cargo metadata uses TOML because Cargo owns that surface. The referenced human-authored policy remains Nickel.

A missing pointer means:

- no project architecture policy load;
- no FCIS or Hexagonal project gate;
- no project architecture receipt;
- no architecture-clean claim.

**Rationale:** Absence must not select Octet-specific paths or imply consent.

### Decision: Validate a typed Nickel policy before source evaluation

**Choice:** `octet-architecture.ncl` declares a schema version, project identity, enforcement mode, production scopes, expected targets, roles, dependency rules, ports, capability classifications, waivers, and non-claims.

The export is deterministic JSON with a BLAKE3 manifest. Runtime Rust consumes only the generated JSON.

**Rationale:** Human composition needs contracts and merge semantics. Runtime behavior needs stable input bytes.

### Decision: Use explicit roles

**Choice:** The policy role vocabulary is:

- `domain-core`;
- `application-shell`;
- `inbound-adapter`;
- `outbound-adapter`;
- `composition-root`;
- `test`;
- `generated`.

Every declared production crate and source scope has exactly one production role. Test and generated scopes remain separate exclusions with explicit coverage.

**Rationale:** A composition root must select adapters without moving that authority into the shell or core.

### Decision: Separate host independence from functional purity

**Choice:** `core_no_std` remains an optional project obligation. It is not a required synonym for `domain-core`.

**Rationale:** `no_std` can support host independence. It does not prove deterministic behavior or absence of FFI, globals, or other effects.

### Decision: Ports belong to the application

**Choice:** Port declarations name an `application_owner_scope`. Adapters implement these ports. Domain-core scopes do not define infrastructure capability ports.

The domain core receives explicit facts, validated policy, actor context, state, time, IDs, and prior observations. It returns decisions, state, events, and effect plans.

**Rationale:** The application owns external capability contracts. Domain meaning stays independent of infrastructure access.

### Decision: Use four project-selected modes

**Choice:** Modes have these effects:

- `inventory` records roles, edges, effects, ports, unknowns, and coverage.
- `advisory` emits architecture findings without promotion blocking.
- `gate` blocks declared structural and FCIS violations.
- `admission` adds transitive effect-closure and stronger evidence obligations.

Malformed policy, missing policy exports, ambiguous scope, and unavailable required inputs are integration errors in every active mode.

**Rationale:** Projects need staged adoption. No active mode can convert missing evidence into a clean result.

### Decision: Remove implicit architecture fallbacks

**Choice:** Consumer evaluation uses only the activated project policy. Octet adds and selects its own explicit self-policy.

Existing global registry entries remain policy definitions and compatibility surfaces. They do not activate a consumer project.

**Rationale:** Explicit self-configuration preserves first-party behavior without ambient consumer assumptions.

### Decision: Govern waivers through the project policy

**Choice:** A waiver binds the stable finding ID, policy identity, source BLAKE3, owner, reason, review condition, and affected claim.

A source or policy identity change makes the waiver stale. A waived result remains `waived`, not `clean`.

Rust `#[allow]` attributes cannot suppress project architecture findings.

**Rationale:** Architecture exceptions affect graph and evidence claims, not one compiler warning site.

### Decision: Keep strict Rust profiles independent

**Choice:** `deterministic-core` continues to enforce its direct lint floor and declared source scope. It does not activate full project architecture policy.

**Rationale:** Existing consumers selected a lint profile, not a new project graph contract.

## Functional Core and Imperative Shell

A pure policy core parses normalized exported values, resolves activation, validates role and scope declarations, and returns policy diagnostics.

The cargo-octet shell reads Cargo metadata, reads the selected export, and passes bounded bytes and workspace facts into the core.

Nickel evaluation and export remain build or operator steps. Runtime Rust does not invoke Nickel.

## Risks / Trade-offs

- Explicit policy adds setup work. A template and inventory mode reduce this cost.
- Role vocabularies can become rigid. Versioned policy and explicit project rules contain that risk.
- Existing fallback tests will change. The explicit Octet self-policy preserves intended first-party coverage.
- Waivers can normalize debt. Visible status, source binding, and review conditions prevent silent permanence.

## Validation

Positive fixtures cover no-policy compatibility, every mode, every role, explicit Octet self-policy, and a current waiver.

Negative fixtures cover escaping pointers, malformed exports, unknown modes, unknown roles, overlapping scopes, uncovered production paths, stale waivers, and architecture claims without activation.