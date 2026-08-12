## Context

`cargo octet check` runs Cargo with the Octet Dylint library. `octet-standards check` separately loads policy, source, and Cargo metadata.

This split lets a consumer run only the lint rail. The standards shell also returns without dependency diagnostics when metadata collection fails.

The project-policy and Architecture IR changes provide explicit activation and compiler facts. This change combines them without merging their claims.

## Decisions

### Decision: Add one optional architecture phase to `cargo octet check`

**Choice:** The runner uses this order:

```text
resolve project configuration
  -> validate architecture activation and exported policy
  -> run Cargo and Dylint
  -> collect Cargo graph and compiler Architecture IR
  -> evaluate architecture policy
  -> run effect closure when mode is admission
  -> combine phase statuses and write artifacts
```

No architecture-policy pointer preserves the current command, child environment, findings, status, and artifact schema.

**Rationale:** One command prevents accidental omission while preserving explicit opt-in.

### Decision: Keep rail authority separate

**Choice:** The parent result contains separate `lint`, `architecture`, and optional `effect_closure` phase records.

Passing one phase never changes another phase status. Architecture findings use policy-check evidence, not static-lint or formal-proof evidence.

**Rationale:** Convenient orchestration must not merge evidence authority.

### Decision: Treat missing evaluation inputs as integration failure

**Choice:** After activation, the runner fails on unreadable policy, failed export integrity, unavailable locked Cargo metadata, incomplete compiler coverage, malformed IR, omitted declared targets, or policy and invocation mismatch.

This rule applies in every active mode. Advisory mode changes finding severity, not input completeness.

**Rationale:** A tool failure cannot become an architecture-clean result.

### Decision: Evaluate a role-aware dependency matrix

**Choice:** The pure graph core enforces these defaults, subject to stricter project rules:

- `domain-core` depends only on declared domain cores and reviewed pure value providers.
- `application-shell` depends inward on domain cores and application-owned contracts.
- `inbound-adapter` depends on the application operation surface and transport providers.
- `outbound-adapter` depends on application-owned ports, domain values, and declared infrastructure providers.
- `composition-root` depends on the application shell and concrete adapters.
- domain cores and application shells never depend on concrete adapter crates.
- only composition roots select concrete adapter implementations.

Test and generated edges follow their explicit policy scopes.

**Rationale:** The composition root resolves concrete infrastructure without contaminating the shell or core.

### Decision: Evaluate FCIS as a set of scoped invariants

**Choice:** The architecture core evaluates:

- complete and disjoint production roles;
- dependency direction and selected feature edges;
- no direct or locally transitive domain-core effects;
- no ambient authority, observability, spawning, synchronization, or resource handles in the domain core;
- no external or infrastructure types in domain and application contracts;
- application-owned inbound and outbound port traits;
- adapter implementations in declared adapter roles;
- concrete adapter construction only in composition roots;
- explicit domain decisions, state changes, events, and effect-plan outputs where policy marks those contracts;
- unknown required compiler facts as blocking in `gate` and `admission`.

Optional `no_std` remains a separate host-independence obligation.

**Rationale:** FCIS is not one path convention or one lint.

### Decision: Use mode-specific finding outcomes

**Choice:**

- `inventory` records architecture observations and findings.
- `advisory` returns findings without architecture promotion failure.
- `gate` blocks hard policy findings and unknown required facts.
- `admission` first satisfies `gate`, then satisfies effect closure for declared core scopes.

A waived finding remains present and has status `waived`.

**Rationale:** Projects can adopt gradually without weakening input integrity.

### Decision: Evaluate exact waivers in the pure core

**Choice:** The evaluator computes stable finding IDs before waiver evaluation.

A waiver applies only when finding, policy, source, owner, review condition, and BLAKE3 bindings match. Stale or broad waivers produce blocking diagnostics.

**Rationale:** A waiver cannot float across changed source or policy.

### Decision: Bind one architecture receipt

**Choice:** The Rust-owned receipt records:

- policy and export identities;
- normalized command and target;
- Cargo lock, metadata, package, feature, and edge identities;
- Architecture IR and coverage identities;
- evaluated roles, ports, capabilities, findings, unknowns, and waivers;
- phase statuses and effect-closure parent when present;
- checker and toolchain identities;
- explicit non-claims;
- final BLAKE3 receipt identity.

Artifact verification recomputes stored identities and links without rerunning Cargo.

**Rationale:** Reviewers need exact evidence for what the architecture gate observed.

### Decision: Preserve existing effect-closure boundaries

**Choice:** `admission` uses the existing effect-closure receipt as an identified child phase. Compiler IR does not silently upgrade source-token closure claims.

**Rationale:** Each analyzer retains its own assumptions and non-claims.

## Functional Core and Imperative Shell

Pure cores resolve role graphs, evaluate invariants, classify modes, apply waivers, and construct receipt values from supplied facts.

The cargo-octet shell reads policy and source files, invokes Cargo, captures metadata, coordinates collector shards, invokes closure, and writes artifacts.

## Adversarial Audit

The gate must reject these cases:

- a caller selects only a compliant package;
- a comment mimics a port trait;
- a renamed import hides an infrastructure type;
- a macro expands to a protected effect;
- an adapter exists only under an inactive feature;
- a factory avoids `new` and `default` names;
- a helper hides a domain-core effect;
- dynamic dispatch remains unresolved;
- Cargo metadata fails;
- one declared target emits no compiler shard;
- a shell constructs a concrete adapter;
- a waiver references old source bytes;
- one clean phase masks another failed phase.

## Risks / Trade-offs

- The integrated run costs more than lint-only checking. Projects select it explicitly and can start in inventory mode.
- Strict unknown handling can block complex macros or dispatch. Findings name the exact unresolved fact and bounded remediation.
- Role policy can misclassify pure providers. Exact provider classification and review remain project-owned.
- Stored integrity is not signer authenticity. Receipts state this non-claim.

## Validation

Positive fixtures cover all roles, valid ports, a composition root, pure core transitions, valid waivers, each mode, and admission composition.

Negative fixtures cover every adversarial case, artifact tampering, malformed policy, policy drift, target drift, feature drift, and architecture overclaims.