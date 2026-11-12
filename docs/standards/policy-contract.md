# Tiger Style Standards Contract

## Purpose

This repo enforces a reusable Rust safety profile through two explicit seams:

- the Dylint lint engine in `src/` and `cargo-octet/`
- the policy registry and standards checker in `standards/` and `crates/octet-standards/`

The machine-readable policy index lives in
[`standards/policies/registry.ncl`](../../standards/policies/registry.ncl). It
links Tiger Style source sections to stable policy IDs, enforcement levels,
current/target enforcement modes, structural mechanisms, proof rails, seed
manifests, checker ownership, suppression requirements, and planned fixture
evidence.

## Functional Core / Imperative Shell

- Standards policy logic MUST live in `crates/octet-standards/src/` as pure
  rule functions over plain inputs.
- Filesystem walking, JSON loading, CLI parsing, and exit-code mapping MUST stay
  in the `octet-standards` binary shell.
- Dylint lint implementation MUST stay in `src/`; standards policy checks MUST
  not be folded into the compiler-plugin crate unless they need rustc HIR.
- `cargo-octet/` MUST remain the lint runner seam for downstream projects.

## Policy Registry

- Policy IDs use the `octet.<pillar>.<rule>` namespace.
- Suppressions use `octet-allow:` comments and MUST cite policy ID, source
  section, owner, rationale, review trigger, and safer default for hard rules.
  A valid suppression covers only diagnostics of its policy at its own path and
  line: `scope=line` (the default) covers the annotated line and
  `scope=next-line` the line after it. Covered diagnostics no longer block;
  the report records them under `suppressed_diagnostics` (markdown
  `## Suppressed Findings`) next to `active_suppressions`. Diagnostics about
  suppressions themselves are never suppressed.
- Hard policies need positive and negative fixtures before becoming blocking.
- Policies that target `structural` MUST export at least one mechanism contract
  such as crate-boundary, capability-api, bounded-primitive, or type-state.
- Crate-boundary mechanisms that declare shell crates drive a
  dependency-direction check over the `cargo metadata --format-version 1`
  crate graph, taken from a `__octet/cargo-metadata.json` source or run from
  the nearest `Cargo.toml` ancestor of a `--source` path. The check fails
  closed with an error diagnostic when no source lies in a Cargo workspace,
  cargo metadata fails, the metadata has no `resolve` graph (as with
  `--no-deps`), or a declared `core_crates`, `shell_crates`, or
  `adapter_crates` entry matches no package or no resolved workspace member.
- Capability-api mechanisms export forbidden ambient authority APIs, declared
  scope classes, capability families, core scopes, shell constructor scopes,
  approved capability names, injected traits/tokens, shell interpreters, and
  deterministic handler evidence so the architecture checker can reject
  core-side ambient effects and emit capability reports from generated policy
  artifacts. The report validates declared policy and observed source shapes
  only; it is not a Rust effect system, transitive purity proof, or
  whole-program runtime-effect absence claim.
- Bounded-primitive mechanisms export forbidden unbounded APIs, approved bounded
  constructors/wrappers, capacity witnesses, and backpressure/overflow witnesses.
- Policies that target `verified` MUST export a proof rail with a command argv,
  exact or globbed source/proof/receipt path mappings, pass signal,
  trust-boundary accounting, verifier mode, receipt-binding mode, and receipt
  provenance obligations. Verification-family checks treat source mappings,
  proof artifact pass signals, and receipt paths as hard artifact obligations:
  by default, a selected verified policy fails unless the declared source, proof,
  and receipt files are present in the checked source tree and the proof artifact
  contains the exact configured pass signal. A receipt that records an
  `exit-status:` line, as `octet-standards check --execute-proof-rails` writes,
  also fails unless the recorded status is `0`; spawn errors and signals fail.
  Rails with
  `receipt_binding = "blake3-command-and-source"` also require command and source
  BLAKE3 bindings matching the configured argv and checked source contents.
  Rails with `receipt_provenance` configured also require matching
  `proof-receipt-schema`, `proof-backend`, `proof-runner`, `proof-environment`,
  and configured `trust-boundary` entries. Rails with `receipt_verifier`
  configured also require exact verifier/toolchain accounting for verifier,
  solver, binary, build closure, Nix derivation, Nix closure, Nix closure graph,
  and toolchain identity. Fixture rails MUST mark `verifier_mode = "fixture"`;
  production Verus rails MUST mark `verifier_mode = "production"` and pin real
  verifier, solver, and toolchain identity. If verifier build-closure, Nix
  closure, or Nix closure graph paths are configured, the corresponding digest is
  derived from checked manifest or graph file contents instead of trusting
  ambient build state; structured Verus receipts record the derived digest, so a
  manifest or graph that no longer hashes to the pinned digest fails receipt
  validation. Before executing a production rail other than an `aeneas-lean`
  rail, whatever its `backend` label, `cargo octet prove` runs the resolved
  verifier with `--identity` and fails the rail as an `unpinned-verifier`
  unless the reported `proof-verifier-binary-blake3`, `proof-solver-blake3`,
  and `proof-toolchain-blake3` equal the pinned digests.
  `cargo octet prove` executes only configured rail argv,
  captures bounded stdout/stderr artifacts, writes structured JSON receipts at
  configured receipt paths, and then emits `formal_proof` provenance only after
  the same pass-signal, binding, provenance, verifier/toolchain, binary, and
  closure gates pass. Command execution is explicit opt-in; read-only receipt
  checking remains the default for consumer safety.
- Proof-relevant seed manifests MUST remain untrusted until a local acceptance
  command or human/oracle review records provenance and re-verification.
- Advisory and human-process-only policies stay visible without pretending static
  tooling can judge them.

## Testing Rules

- Test suites MUST include both positive and negative cases.
- Standards checker tests MUST exercise pure rule functions directly.
- Fixture rails MUST prove compliant and noncompliant examples separately.
- Error-path tests MUST prove the exact failure surface: returned error variant,
  message, and no hidden panic.

## Numeric and Unsafe Discipline

- Non-obvious numeric literals MUST be named constants, parameters, or config
  values.
- Every unsafe block MUST be documented, or unsafe MUST be absent.
- Core validation helpers SHOULD prefer named input structs over ambiguous
  same-typed parameter lists.

## Verification Commands

Run these inside `nix develop` or through `nix develop path:. -c ...`:

```bash
cargo test -p octet-standards
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
nix run .#octet-policy-validate
nix run .#octet-standards -- --help
nix build .#checks.x86_64-linux.policy-registry --no-link
nix build .#checks.x86_64-linux.standards-fixtures --no-link
nix run .#cargo-octet -- check
```

## Policy clauses

These clauses describe Octet's existing policy declarations. They do not
reproduce the withdrawn standards narrative or grant permission over it. The
Nickel registry retains historical citation labels in comments; its live
`source_path`, `source_heading`, and `source_anchor` fields point here.

A clause states the rule, not a claim that every target mechanism is currently
implemented or verified. Applicability, classification, current and target
enforcement modes, checker ownership, fixtures, suppressions, and proof rails
remain explicit in `standards/policies/registry.ncl`. Human-process and advisory
clauses do not become blocking static checks merely by appearing here.

### tigerstyle-architecture-fcis-crate-boundary

`tigerstyle.architecture.fcis-crate-boundary`: Business logic belongs behind compiler-visible crate boundaries with an FCIS core/shell split.

### tigerstyle-architecture-external-domain-leakage

`tigerstyle.architecture.external-domain-leakage`: External domain types must not leak into core domain APIs.

### tigerstyle-architecture-explicit-boundary-conversions

`tigerstyle.architecture.explicit-boundary-conversions`: Cross-context data translation must be explicit at adapters instead of implicit inside domain code.

### tigerstyle-architecture-root-crate-thinness

`tigerstyle.architecture.root-crate-thinness`: Root package stays a thin compatibility and binary wiring layer.

### tigerstyle-architecture-public-api-budget

`tigerstyle.architecture.public-api-budget`: Public API surface expansion should be controlled and reviewable.

### tigerstyle-devtools-single-command-verification

`tigerstyle.devtools.single-command-verification`: The standard verification set must run through a documented local-first command.

### tigerstyle-devtools-deny-warnings

`tigerstyle.devtools.deny-warnings`: Warnings must fail verification instead of serving as optional advice.

### tigerstyle-devtools-formatting

`tigerstyle.devtools.formatting`: Formatting must be enforced by a local-first formatting gate.

### tigerstyle-devtools-documentation-gates

`tigerstyle.devtools.documentation-gates`: Public APIs and standards need documentation gates where the signal is reliable.

### tigerstyle-devtools-dependency-audit

`tigerstyle.devtools.dependency-audit`: Dependency advisories, license policy, and duplicate dependency risk must be visible in local verification.

### tigerstyle-devtools-lint-suppressions

`tigerstyle.devtools.lint-suppressions`: Lint suppressions must be narrow and justified by policy metadata.

### tigerstyle-embedded-opt-in-subsystem-manifest

`tigerstyle.embedded.opt-in-subsystem-manifest`: Hardware subsystems must start only when declared by a hardware manifest or explicit config.

### tigerstyle-embedded-bounded-queues

`tigerstyle.embedded.bounded-queues`: Queues and channels at the edge must be bounded.

### tigerstyle-embedded-blocking-io-timeouts

`tigerstyle.embedded.blocking-io-timeouts`: Blocking I/O paths must carry explicit timeout or cancellation bounds.

### tigerstyle-embedded-safe-serial-api

`tigerstyle.embedded.safe-serial-api`: Serial APIs must expose safe, bounded operations instead of leaking raw unsafe or unchecked byte handling.

### tigerstyle-embedded-bounded-task-topology

`tigerstyle.embedded.bounded-task-topology`: Task and thread topology must be bounded instead of growing dynamically without limit.

### tigerstyle-resilience-typed-errors

`tigerstyle.resilience.typed-errors`: Failure modes must be modeled as explicit typed errors instead of catch-all or string buckets.

### tigerstyle-resilience-no-silent-defaults

`tigerstyle.resilience.no-silent-defaults`: Fallible paths must not silently default away errors.

### tigerstyle-resilience-modeled-failure-modes

`tigerstyle.resilience.modeled-failure-modes`: Known operational failures must be represented explicitly instead of hidden behind vague buckets.

### tigerstyle-resilience-durable-persistence-claims

`tigerstyle.resilience.durable-persistence-claims`: Declared durable boundaries observe synchronization, configured commit, or adapter results and never treat Write::flush() as durable-persistence evidence.

### tigerstyle-resilience-fail-closed-handling

`tigerstyle.resilience.fail-closed-handling`: Error handling must fail closed when safety or correctness state is uncertain.

### tigerstyle-resilience-five-whys-process

`tigerstyle.resilience.five-whys-process`: Incident analysis should use Five Whys to reach architectural causes without assigning blame.

### tigerstyle-orchestration-transition-tables

`tigerstyle.orchestration.transition-tables`: State-machine transitions should be declared in tables that the compiler or checker can inspect.

### tigerstyle-orchestration-deny-by-default-work-acceptance

`tigerstyle.orchestration.deny-by-default-work-acceptance`: Work acceptance gates must be deny-by-default allow lists.

### tigerstyle-orchestration-invalid-transition-preserves-state

`tigerstyle.orchestration.invalid-transition-preserves-state`: Rejected state-machine transitions must leave state unchanged.

### tigerstyle-orchestration-priority-ordering

`tigerstyle.orchestration.priority-ordering`: Urgent, expedite, and standard queues must preserve configured priority and FIFO within a tier.

### tigerstyle-idioms-one-behavior-one-type

`tigerstyle.idioms.one-behavior-one-type`: Duplicated primitive behavior should become a named type that carries the shared invariant.

### tigerstyle-idioms-duplicate-primitive-behavior

`tigerstyle.idioms.duplicate-primitive-behavior`: Repeated primitive validation/conversion behavior should be reported for possible newtype extraction.

### tigerstyle-idioms-thin-forwarder-verifier

`tigerstyle.idioms.thin-forwarder-verifier`: Approved idioms such as thin forwarders must be recognized by verifiers instead of broadly suppressed.

### tigerstyle-external-source-classification

`tigerstyle.external.source-classification`: External concepts must be classified by source, Tiger Style outcome, enforcement tier, and first implementation surface before incorporation.

### tigerstyle-devtools-dynamic-evidence-profiles

`tigerstyle.devtools.dynamic-evidence-profiles`: Runtime and profiling tools are selected opt-in evidence profiles with run timing, acceptance signals, and saved transcripts.

### tigerstyle-idioms-imported-rust-idioms

`tigerstyle.idioms.imported-rust-idioms`: Imported Rust idioms must map to Tiger Style outcomes and start as docs/advisory policy unless fixtures justify a hard gate.

### tigerstyle-idioms-functional-api-boundaries

`tigerstyle.idioms.functional-api-boundaries`: Functional API guidance should promote type-level state/protocol separation while rejecting opaque point-free or deeply chained hot-path code.

### tigerstyle-build-what-you-test-test-builders

`tigerstyle.build-what-you-test.test-builders`: Broad test configuration setup should use builders that expose only relevant preconditions.

### tigerstyle-build-what-you-test-visible-preconditions

`tigerstyle.build-what-you-test.visible-preconditions`: Tests must state meaningful preconditions instead of hiding them in broad fixture setup.

### tigerstyle-build-what-you-test-no-hidden-external-fixtures

`tigerstyle.build-what-you-test.no-hidden-external-fixtures`: Unit tests must not depend on hidden external resources or setup scripts.

### tigerstyle-build-what-you-test-process-global-isolation

`tigerstyle.build-what-you-test.process-global-isolation`: Tests must not couple through process-global state unless isolated or serialized.

### tigerstyle-metrics-flat-telemetry

`tigerstyle.metrics.flat-telemetry`: Health and telemetry payloads should use flat fields instead of nested trees.

### tigerstyle-metrics-threshold-backed-health

`tigerstyle.metrics.threshold-backed-health`: Health checks must use named thresholds and explicit pass/fail boundaries.

### tigerstyle-metrics-prometheus-text

`tigerstyle.metrics.prometheus-text`: Metrics endpoints that expose Prometheus must satisfy text exposition fixtures.

### tigerstyle-metrics-blake3-audit-chain

`tigerstyle.metrics.blake3-audit-chain`: Audit chains should use BLAKE3-linked records with tamper detection fixtures.

### tigerstyle-verification-positive-negative-tests

`tigerstyle.verification.positive-negative-tests`: Test suites must include positive and negative cases.

### tigerstyle-verification-first-properties

`tigerstyle.verification.first-properties`: Tests should satisfy FIRST properties where statically or fixture-detectable.

### tigerstyle-verification-exact-error-assertions

`tigerstyle.verification.exact-error-assertions`: Error-path tests must assert exact returned error surfaces instead of logs or vague failure.

### tigerstyle-verification-target-hardware-coverage

`tigerstyle.verification.target-hardware-coverage`: Tests that claim hardware-relevant safety evidence must declare target or hardware coverage.

### tigerstyle-verification-safety-critical-claims

`tigerstyle.verification.safety-critical-claims`: Safety-critical claims must cite standard, level or class, evidence path, tool-qualification notes, and verification strategy.

### tigerstyle-fitness-readability

`tigerstyle.fitness.readability`: Readability fitness should be reported from deterministic source-shape inputs.

### tigerstyle-fitness-changeability

`tigerstyle.fitness.changeability`: Changeability fitness should report change coupling and boundary pressure.

### tigerstyle-fitness-operational-resilience

`tigerstyle.fitness.operational-resilience`: Operational resilience fitness should summarize observable failure-handling and recovery signals.

### tigerstyle-fitness-dependency-health

`tigerstyle.fitness.dependency-health`: Dependency health fitness should report age, advisories, duplication, and license risk.

### tigerstyle-fitness-test-quality

`tigerstyle.fitness.test-quality`: Test quality fitness should report error-path coverage and fixture health signals.

### tigerstyle-fitness-change-risk

`tigerstyle.fitness.change-risk`: Change risk fitness should report coupled files and risky churn trends when history exists.

### tigerstyle-culture-flow-protection

`tigerstyle.culture.flow-protection`: Teams should protect deep-work flow and reduce avoidable context switching.

### tigerstyle-culture-snapshot-readability

`tigerstyle.culture.snapshot-readability`: System structure should support fast, accurate mental snapshots for readers.

### tigerstyle-architecture-effect-capability-boundaries

`tigerstyle.architecture.effect-capability-boundaries`: Domain cores receive explicit facts and return effect plans while application shells interpret ambient authority.

### tigerstyle-architecture-hexagonal-ports

`tigerstyle.architecture.hexagonal-ports`: Declared ports remain application-owned traits with adapter-owned concrete implementations.

### tigerstyle-architecture-top-level-purity

`tigerstyle.architecture.top-level-purity`: Constructors, builders, statics, and lazy initialization validate or assemble delayed effects instead of performing ambient IO.

### tigerstyle-devtools-content-addressed-evidence

`tigerstyle.devtools.content-addressed-evidence`: Evidence, findings, transcripts, and selected function fingerprints use BLAKE3 identity hashes without claiming correctness.

### tigerstyle-devtools-live-example-verification

`tigerstyle.devtools.live-example-verification`: Verification-claiming examples link to fixtures, doctests, trybuild cases, or saved transcripts with content hashes.

### tigerstyle-idioms-nominal-type-boundaries

`tigerstyle.idioms.nominal-type-boundaries`: Identities, units, secrets, capabilities, protocol states, and idempotency keys use nominal wrappers instead of primitive aliases.

### tigerstyle-idioms-named-capability-vocabulary

`tigerstyle.idioms.named-capability-vocabulary`: Capability traits and tokens are named by the authority they grant rather than vague context/client names.

### tigerstyle-verification-deterministic-effect-handlers

`tigerstyle.verification.deterministic-effect-handlers`: Effect handlers used for tests record requested effects and replay scripted time, filesystem, network, and environment responses.

### octet-aeneas_lean_proof_rails-fixture

`octet.aeneas_lean_proof_rails.fixture`: An opt-in pinned Aeneas/Charon/Lean fixture rail binds a real generated model, named theorem, receipt, and artifact identities without broadening starter checks.

### octet-vermilion_canary_rail-recorded

`octet.vermilion_canary_rail.recorded`: The measured Vermilion u64 canary is validated as recorded-only evidence and emitted as a manual-review proof card; it is never promoted to a formal-proof claim.

### octet-aeneas_consumer_contract-external-fixture

`octet.aeneas_consumer_contract.external-fixture`: An external-consumer fixture binds exact source, selected item, generated model, maintained theorem, requirement, trust closure, and provider identities.

### octet-charon_semantic_analysis-recorded-snapshot

`octet.charon_semantic_analysis.recorded-snapshot`: An opt-in pinned Charon rail projects selected LLBC facts and conservative effect candidates as role-separated recorded evidence without proof-authority promotion.

### octet-verus_proof_rails-addition-bound

`octet.verus_proof_rails.addition-bound`: A fixture-scoped Verus rail binds the Octet-owned proof target, pinned verifier wrapper identity, pass signal, receipt, and non-claim boundary.

### octet-verus_proof_rails-addition-bound-production

`octet.verus_proof_rails.addition-bound-production`: A production-mode Verus pilot rail runs the pinned upstream Verus binary release against the Octet-owned addition-bound proof target.

### octet-verus_proof_rails-ordering-reflexive-production

`octet.verus_proof_rails.ordering-reflexive-production`: A second production-mode Verus pilot rail runs the pinned upstream Verus binary release against the Octet-owned reflexive-order proof target.

### octet-verus_proof_rails-addition-bound-production-negative

`octet.verus_proof_rails.addition-bound-production-negative`: An opt-in production-mode Verus negative rail proves invalid proofs fail closed and emit no formal_proof provenance.

### tigerstyle-resilience-remote-effect-contracts

`tigerstyle.resilience.remote-effect-contracts`: Remote boundaries name endpoint authority, timeout, retry, idempotency key, receipt identity, and replay evidence.

### tigerstyle-resilience-semantic-refactor-evidence

`tigerstyle.resilience.semantic-refactor-evidence`: Behavior-preserving moves and renames record old/new symbol identity, rationale, fingerprints, and verification command.


