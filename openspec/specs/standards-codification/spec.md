# standards-codification Specification

## Purpose

This spec defines the Tiger Style standards policy registry, standards-checker command surface, fixture expectations, suppression metadata, verification rails, and naming hygiene needed to keep codified standards traceable to their source material.
## Requirements
### Requirement: Tiger Style Policy Registry

The system MUST maintain a machine-readable Tiger Style policy registry that traces each codification candidate to `docs/standards/source/tigerstyle-standards.adoc` or an included `docs/standards/source/*.adoc` source section.

Each policy entry MUST include a stable `tigerstyle.*` policy ID, pillar, source path, source heading, summary, enforcement level, checker owner, default severity, suppression requirements, and verification evidence requirements.

#### Scenario: Codifiable source becomes policy

- GIVEN a Tiger Style source section states a repeatable rule such as "every unbounded queue is a latent OOM"
- WHEN the policy inventory is generated or updated
- THEN the registry contains a policy entry linked to `docs/standards/source/embedded.adoc`
- AND the entry classifies whether the rule is hard, advisory, or human-process-only
- AND the entry names the checker or planned checker that owns enforcement

#### Scenario: Human-only guidance stays explicit

- GIVEN a Tiger Style source section describes flow protection, regulatory classification, or Five Whys facilitation
- WHEN the policy inventory is reviewed
- THEN the registry marks the entry as human-process-only or advisory
- AND no blocking build rule is created without a separate, measurable enforcement design

#### Scenario: Missing traceability is rejected

- GIVEN a policy entry lacks a source path or source heading
- WHEN the registry is validated
- THEN validation fails with a diagnostic naming the incomplete policy entry

### Requirement: Typed Policy Configuration

The system MUST use typed Nickel contracts for project-owned Tiger Style policy configuration, with generated machine formats used only as tool inputs.

The policy schema MUST reject entries with missing required fields, unknown enforcement levels, missing suppression metadata, or checker references that do not map to a known checker.

#### Scenario: Valid Nickel policy exports for tools

- GIVEN a policy registry that satisfies the Nickel contract
- WHEN the export command runs
- THEN it produces deterministic tool input data
- AND the exported data preserves policy IDs, source links, enforcement levels, and checker ownership

#### Scenario: Invalid policy fails before checks run

- GIVEN a policy entry declares an unknown enforcement level
- WHEN registry validation runs
- THEN the command fails before running source-code checks
- AND the diagnostic identifies the invalid field and policy ID

### Requirement: Reusable Tiger Style Standards Runner

The system MUST provide a reusable `tigerstyle-standards` checker command with a pure rule-evaluation core and a thin imperative shell for filesystem traversal, CLI parsing, and process exit codes.

The runner MUST support selecting rule families, validating fixtures, emitting stable human-readable diagnostics, and producing machine-readable output for CI dashboards.

#### Scenario: Blocking hard rule fails deterministically

- GIVEN a fixture that violates a hard policy
- WHEN the checker runs against that fixture
- THEN the command exits non-zero
- AND the diagnostic includes policy ID, file path, line when available, source section, and remediation summary

#### Scenario: Advisory rule reports without blocking

- GIVEN a fixture that only violates advisory policies
- WHEN the checker runs in default CI mode
- THEN the command exits successfully
- AND the report includes the advisory finding with its policy ID and source section

### Requirement: Hard Rule Families

The system MUST codify high-confidence Tiger Style rules as hard-rule families when the rule can be checked with low false-positive risk and clear remediation.

At minimum, the policy plan MUST cover these hard-rule candidates:

- Architecture: FCIS crate boundary, no external domain type leakage into core, explicit conversion at boundaries, root crate thinness, and controlled public API surface.
- DevTools: single-command verification, deny warnings, formatting enforcement, documentation gates, dependency audit gates, and no unexplained lint suppressions.
- Embedded: bounded channels and queues, timeout-backed blocking I/O, opt-in subsystem startup from a hardware manifest, safe serial APIs, and bounded task/thread topology.
- Resilience: explicit typed errors, no silent defaulting on fallible paths, modeled failure modes instead of catch-all buckets, and fail-closed error handling.
- Orchestration: declarative transition tables, deny-by-default work acceptance, invalid transitions that leave state unchanged, and priority ordering properties.
- Idioms: one behavior / one type extraction candidates, duplicate primitive-behavior detection, and verifier recognition of approved thin-forwarder idiom output.
- Build What You Test: explicit test builders for broad config structs, visible preconditions, no hidden external fixtures in unit tests, and isolated process-global state.
- Metrics: flat telemetry payloads, threshold-backed health checks, Prometheus text exposition when metrics endpoints exist, and BLAKE3 hash-chain audit integrity.
- Verification: positive and negative tests, FIRST properties where statically checkable, target/hardware coverage declarations, and safety-critical evidence prompts.

#### Scenario: Candidate rule with weak signal becomes advisory

- GIVEN a candidate rule cannot distinguish safe code from unsafe code without human context
- WHEN the hard-rule family is reviewed
- THEN the registry classifies it as advisory or human-process-only
- AND the design records why it is not a blocking gate yet

#### Scenario: Hard rule has fixture evidence

- GIVEN a hard-rule policy is enabled
- WHEN fixture validation runs
- THEN at least one passing fixture proves the checker accepts the intended shape
- AND at least one failing fixture proves the checker rejects the unsafe or noncompliant shape

### Requirement: Suppressions and Escape Hatches

The system MUST make any suppression of a Tiger Style hard rule explicit, traceable, and narrow.

Suppressions MUST use `tigerstyle-allow:` and include policy ID, source section, justification, owner, review trigger, and a reference to the safer default being bypassed. Blanket allow-lists that hide future true positives MUST be rejected.

#### Scenario: Narrow suppression is accepted

- GIVEN a hardware-bound module needs a justified exception to a hard rule
- WHEN the suppression includes policy ID, source section, owner, rationale, review trigger, and the safer default being bypassed
- THEN the checker accepts only the annotated location
- AND the report records the active suppression

#### Scenario: Blanket suppression is rejected

- GIVEN a suppression disables a rule across a module or workspace without policy ID and owner
- WHEN the checker runs
- THEN the command fails
- AND the diagnostic points to the suppression rule and remediation format

### Requirement: Verification Rail Integration

The system MUST integrate Tiger Style standards codification with existing verification rails through local-first commands and flake checks.

The command set MUST remain runnable from the Nix dev shell and MUST include standards checker self-tests, policy registry validation, fixture validation, workspace checks, Clippy denial, and Tiger Style lint checks as separate seams.

#### Scenario: Local-first standards verification succeeds

- GIVEN a developer enters the Nix dev shell or invokes `nix develop path:. -c ...`
- WHEN the documented Tiger Style standards verification commands run
- THEN they execute standards self-tests, policy validation, fixture checks, formatting, and workspace checks with repo-local target directories where Cargo artifacts are involved
- AND they exit successfully for a compliant workspace

#### Scenario: Fixture regression blocks CI

- GIVEN a change weakens a checker so a failing fixture is no longer reported
- WHEN fixture validation runs under the flake check
- THEN CI fails with the fixture name, expected policy ID, and missing diagnostic

### Requirement: Rename Hygiene

The system MUST remove legacy project naming from Tiger Style-owned standards artifacts.

The migrated standards source, policy IDs, suppression marker, checker command, docs, Nix apps, and OpenSpec artifacts MUST use Tiger Style naming.

#### Scenario: Legacy project names are absent from Tiger Style standards artifacts

- GIVEN the standards migration is staged in the `tigerstyle` repo
- WHEN a case-insensitive source scan runs over Rust, Markdown, Nickel, AsciiDoc, Nix, TOML, YAML, and text artifacts
- THEN it finds no legacy project strings strings in the migrated Tiger Style standards surface

### Requirement: External Source Incorporation Classification [r[external-source-incorporation-classification]]

The system MUST classify each incorporated external reference concept before it becomes Tigerstyle standards guidance, policy registry data, or lint scope.

Each classification MUST name the external source, the imported concept, the Tiger Style outcome it supports, the enforcement tier, and whether the first implementation is documentation, policy metadata, a checker, or a human checklist.

#### Scenario: External concepts are classified before incorporation [r[external-source-incorporation-classification.scenario.classified-before-incorporation]]

- GIVEN a contributor proposes importing a Valgrind tool-suite concept or a Rust Design Patterns idiom
- WHEN the standards incorporation plan is updated
- THEN the plan records the source URL, selected concept, Tiger Style rationale, and enforcement tier
- AND concepts with weak static signal remain advisory or human-process-only until separate fixture evidence exists

### Requirement: Dynamic Evidence Profiles [r[dynamic-evidence-profiles]]

The system MUST document opt-in dynamic evidence profiles for bug classes that static Tigerstyle lints cannot fully prove.

The profiles MUST cover memory/allocation behavior, cache or hot-path profiling, race/concurrency analysis, unsafe or FFI boundary checking, and release-readiness timing. Each profile MUST state when to run it, the expected acceptance signal, and how evidence is saved for OpenSpec review.

#### Scenario: Dynamic profile guidance names run timing and acceptance signal [r[dynamic-evidence-profiles.scenario.timing-and-acceptance]]

- GIVEN a maintainer selects an unsafe, FFI, concurrency, or performance-sensitive slice
- WHEN the relevant dynamic evidence profile is read
- THEN the profile names the applicable run timing such as after a big change, suspected bug, pre-release, or focused CI rail
- AND it names the acceptance signal or saved transcript shape required for review

#### Scenario: Heavyweight tools remain optional until a profile is selected [r[dynamic-evidence-profiles.scenario.optional-until-selected]]

- GIVEN a normal Tigerstyle consumer runs the default lint or standards checks
- WHEN no dynamic evidence profile is selected
- THEN the default command set does not require Valgrind, sanitizers, heap profilers, cache profilers, Miri, or Loom to be installed
- AND profile docs explain how to run selected tools only for applicable evidence slices

### Requirement: Rust Idiom Guidance [r[rust-idiom-guidance]]

The system MUST provide Tigerstyle-owned Rust idiom guidance that imports only patterns that improve safety, explicitness, boundedness, recoverability, performance evidence, or developer comprehension.

The guidance MUST cover newtypes for units/capabilities/secrets, RAII resource guards, contained unsafe modules, borrowed argument types, temporary mutability, clone-to-satisfy-borrow-checker alternatives, returning consumed arguments on error, constrained builders, command/effects-as-data, FFI wrappers and error mapping, restrained `Deref` use, small-crate boundaries, and functional API patterns that use generics or sealed traits to make invalid protocol/state operations unrepresentable.

#### Scenario: Rust idiom guidance maps each imported pattern to Tiger Style outcomes [r[rust-idiom-guidance.scenario.maps-patterns-to-outcomes]]

- GIVEN a reader opens the Rust idiom guidance
- WHEN they review an imported pattern or anti-pattern
- THEN the guidance states the Tiger Style outcome it supports
- AND it distinguishes immediate documentation/checklist guidance from future lint or policy enforcement

#### Scenario: Future lints require fixtures before hard gating [r[rust-idiom-guidance.scenario.future-lints-require-fixtures]]

- GIVEN a pattern-derived lint candidate such as borrowed argument types, unsafe module containment, clone-avoidance, temporary mutability, or `Deref` restraint is proposed
- WHEN the policy registry classifies it as a hard rule
- THEN positive and negative fixtures prove the intended compliant and noncompliant shapes
- AND the lint is not promoted to a default hard gate without that fixture evidence

### Requirement: Functional API Boundary Guidance [r[functional-api-boundary-guidance]]

The system MUST document which functional programming concepts from Rust Design Patterns are acceptable Tigerstyle guidance and which remain non-goals.

The guidance MUST promote type-class-style generics, sealed traits, associated types, and marker-state APIs when they encode protocol, state, capability, or format differences at compile time. It MUST treat optics, visitors, and prism-like adapter splits as advanced human-review guidance for serialization, traversal, and plugin boundaries rather than default application style. It MUST caution against opaque point-free or deeply chained functional code in hot paths, safety-critical paths, and code that needs dense assertions.

#### Scenario: Functional API guidance promotes type-level state separation without opaque cleverness [r[functional-api-boundary-guidance.scenario.type-level-state-not-cleverness]]

- GIVEN a Tigerstyle guide imports functional programming concepts
- WHEN a reader compares type-class/generic-state APIs, optics/visitor adapter APIs, and fold-heavy declarative code
- THEN the guide recommends type-level state or protocol separation where it removes runtime `Option`/enum checks from known-state code paths
- AND it classifies optics or visitor/prism patterns as advanced boundary tools requiring human design review
- AND it warns that long iterator chains or point-free transformations are inappropriate when they hide bounds, branches, assertions, or hot-loop costs

### Requirement: Function Fingerprint Policy [r[function-fingerprint-policy]]

The system MUST define a Tigerstyle policy for selective content-addressed function fingerprints that bind safety-critical Rust functions to exact reviewed or verified source-token bodies.

The policy MUST state that fingerprints are identity and review gates, not correctness proofs. It MUST recommend fingerprints for verified primitives, protocol transitions, security-sensitive functions, capability/token constructors, safety-critical arithmetic, and fixture-backed documentation examples. It MUST NOT require blanket fingerprints for every function in a workspace.

#### Scenario: Critical function categories are documented [r[function-fingerprint-policy.scenario.critical-categories]]

- GIVEN a maintainer reads the function fingerprint guidance
- WHEN they decide whether a function should be fingerprinted
- THEN the guidance names verified primitives, protocol transitions, security-sensitive functions, capability/token constructors, safety-critical arithmetic, and fixture-backed examples as recommended categories
- AND the guidance says ordinary functions do not need fingerprints by default

#### Scenario: Fingerprints do not claim correctness [r[function-fingerprint-policy.scenario.identity-not-correctness]]

- GIVEN a fingerprinted function matches its stored hash
- WHEN the tool reports the fingerprint as valid
- THEN the documentation and diagnostic language state that the exact source body matches the reviewed body
- AND they do not claim the function is semantically correct, safe, or verified solely because the hash matches

### Requirement: Function Fingerprint Metadata [r[function-fingerprint-metadata]]

The system MUST require review metadata for function fingerprints so each selected function records why content identity matters.

At minimum, each fingerprint MUST name the hash value and review class. Supported review classes MUST include `verified`, `protocol-transition`, `security`, `capability-token`, `safety-arithmetic`, and `example-evidence`. Optional metadata MAY include owner, spec ID, evidence path, and macro allowance.

#### Scenario: Missing review class is rejected [r[function-fingerprint-metadata.scenario.missing-review-class]]

- GIVEN a function carries a fingerprint hash but omits review class metadata
- WHEN fingerprint validation runs in strict mode
- THEN validation reports the function as incomplete
- AND the diagnostic names the missing review class field

#### Scenario: Review metadata links to evidence [r[function-fingerprint-metadata.scenario.links-to-evidence]]

- GIVEN a fingerprinted verified primitive or protocol transition names a spec ID or evidence path
- WHEN fingerprint inventory output is generated
- THEN the output preserves the function name, hash, review class, and evidence reference
- AND reviewers can trace the function identity to the relevant OpenSpec or verification artifact

### Requirement: Function Fingerprint Hash Semantics [r[function-fingerprint-hash-semantics]]

The system MUST compute function fingerprints from a versioned, domain-separated, normalized representation of Rust function source tokens.

The hash input MUST include normalized visibility, signature, generic parameters, where clauses, and body tokens. It MUST exclude the fingerprint attribute or marker itself, whitespace, comments, file paths, line numbers, timestamps, local usernames, and absolute paths. The hash algorithm MUST be identified with a `b3:` prefix for BLAKE3 values.

#### Scenario: Formatting and comments do not change hash [r[function-fingerprint-hash-semantics.scenario.formatting-stable]]

- GIVEN a fingerprinted function whose executable tokens are unchanged
- WHEN only rustfmt layout, comments, or line numbers change
- THEN recomputing the fingerprint yields the same `b3:` value

#### Scenario: Signature or body token changes update hash [r[function-fingerprint-hash-semantics.scenario.token-change-detected]]

- GIVEN a fingerprinted function with a stored hash
- WHEN its signature, generics, where clause, visibility, or body tokens change
- THEN fingerprint validation reports a mismatch with expected and actual `b3:` values

### Requirement: Function Fingerprint Macro Boundary [r[function-fingerprint-macro-boundary]]

The system MUST make macro-call limitations explicit for fingerprinted functions.

If a fingerprinted function contains source macro invocations, validation MUST report a macro-boundary warning unless the fingerprint metadata explicitly allows macros. The warning MUST explain that source-token fingerprints do not cover macro definition changes or expansion semantics.

#### Scenario: Macro call warns by default [r[function-fingerprint-macro-boundary.scenario.warns-by-default]]

- GIVEN a fingerprinted function body contains a macro invocation
- WHEN fingerprint validation runs
- THEN validation emits a macro-boundary diagnostic
- AND the diagnostic explains that macro definitions are not covered by the function source-token hash

#### Scenario: Explicit macro allowance suppresses caveat [r[function-fingerprint-macro-boundary.scenario.explicit-allowance]]

- GIVEN a fingerprinted function body contains a macro invocation and metadata explicitly allows macros
- WHEN fingerprint validation runs
- THEN validation does not emit the default macro-boundary warning
- AND inventory output preserves that macro allowance for review

### Requirement: Effect Capability Boundary Guidance [r[effect-capability-boundary-guidance]]

The system MUST document a Rust effect-capability boundary model that keeps domain core logic free of ambient IO, time, randomness, env access, process spawning, networking, and remote execution while allowing effects to appear as explicit data or named capability traits.

#### Scenario: Core effects are explicit [r[effect-capability-boundary-guidance-scenario-core-effects-are-explicit]]

- GIVEN a developer designs a core function
- WHEN they consult the guidance
- THEN it describes returning effect descriptions or accepting named capabilities instead of reading ambient state directly

#### Scenario: Shell interprets capabilities [r[effect-capability-boundary-guidance-scenario-shell-interprets-capabilities]]

- GIVEN core logic emits an effect description
- WHEN shell code executes the program
- THEN the shell owns IO, time, randomness, network, process, and remote side effects

### Requirement: Future effect lint classification [r[future-effect-lint-classification]]

The system MUST classify future effect-boundary lint candidates before hard gating them, including effectful globals, effectful constructors, and expanded pure-module checks.

#### Scenario: Lint candidates stay staged [r[future-effect-lint-classification-scenario-lint-candidates-stay-staged]]

- GIVEN an effect-boundary checker has weak static signal
- WHEN policy metadata is reviewed
- THEN the checker remains advisory or human-process-only until positive and negative fixtures prove low false-positive behavior

### Requirement: Top Level Purity Guidance [r[top-level-purity-guidance]]

The system MUST document that Rust globals, static initializers, lazy initializers, constructors, Default implementations, and builders should not hide filesystem, env, clock, randomness, network, process, or spawn effects.

#### Scenario: Initializer side effects are rejected by guidance [r[top-level-purity-guidance-scenario-initializer-side-effects-are-rejected-by-guidance]]

- GIVEN a LazyLock initializer reads env or files
- WHEN a reviewer applies the guidance
- THEN it is classified as shell work or delayed effect construction rather than pure initialization

#### Scenario: Constructors validate only [r[top-level-purity-guidance-scenario-constructors-validate-only]]

- GIVEN a `new`, `default`, or `build` function exists in core logic
- WHEN it is reviewed
- THEN the accepted shape validates and constructs values without performing ambient effects

### Requirement: Delayed Effect Pattern [r[delayed-effect-pattern]]

The system MUST document delayed-effect alternatives such as returning commands, plans, tasks, or capability requests for shell interpretation.

#### Scenario: Effect is represented as data [r[delayed-effect-pattern-scenario-effect-is-represented-as-data]]

- GIVEN construction needs future IO
- WHEN the code follows the delayed-effect pattern
- THEN it returns an explicit effect description and leaves execution to shell code

### Requirement: Remote Execution Effect Contract [r[remote-execution-effect-contract]]

The system MUST document remote execution as a first-class effect family that requires explicit timeout, retry bound, batch bound, idempotency key, and receipt or replay semantics.

#### Scenario: Remote task carries bounds [r[remote-execution-effect-contract-scenario-remote-task-carries-bounds]]

- GIVEN a design schedules remote work
- WHEN the remote effect contract is applied
- THEN the task model names timeout, max retries, idempotency key, and batch bound

#### Scenario: Receipt semantics are explicit [r[remote-execution-effect-contract-scenario-receipt-semantics-are-explicit]]

- GIVEN remote work completes or fails
- WHEN review evidence is captured
- THEN the receipt records enough identity to replay, correlate, or audit the attempted work

### Requirement: Remote Lint Staging [r[remote-lint-staging]]

The system MUST keep remote-effect enforcement advisory or checklist-based until fixtures prove a low false-positive static signal.

#### Scenario: Weak static signal remains human reviewed [r[remote-lint-staging-scenario-weak-static-signal-remains-human-reviewed]]

- GIVEN remote execution APIs differ by consumer
- WHEN a checker is proposed
- THEN the policy records human-process or advisory classification before hard gating

### Requirement: Named Capability Vocabulary [r[named-capability-vocabulary]]

The system MUST document a Rust capability vocabulary pattern for domain effects using explicit traits, tokens, or effect enums with names that expose authority and lifecycle obligations.

#### Scenario: Capability name exposes authority [r[named-capability-vocabulary-scenario-capability-name-exposes-authority]]

- GIVEN code needs clock, entropy, credential, object-store, or remote authority
- WHEN the capability is modeled
- THEN its name and API expose the authority being granted rather than hiding it behind generic context objects

#### Scenario: Core admissibility is explicit [r[named-capability-vocabulary-scenario-core-admissibility-is-explicit]]

- GIVEN a capability crosses into core-adjacent logic
- WHEN it is reviewed
- THEN guidance distinguishes pure data capabilities from shell-only effectful capabilities

### Requirement: Deterministic Capability Handlers [r[deterministic-capability-handlers]]

The system MUST require examples for capability traits to include deterministic test handlers such as fixed clocks, seeded entropy, recording stores, or fake remote executors.

#### Scenario: Test handler avoids ambient state [r[deterministic-capability-handlers-scenario-test-handler-avoids-ambient-state]]

- GIVEN a capability example appears in docs
- WHEN the test form is shown
- THEN it uses deterministic handler state rather than ambient time, randomness, network, or filesystem

### Requirement: Nominal Type Boundary Guidance [r[nominal-type-boundary-guidance]]

The system MUST document when Rust code should use nominal newtypes instead of interchangeable primitive or structural types for IDs, units, secrets, capability tokens, protocol states, and authority-bearing values.

For declared type-state protocol surfaces, the system MUST export structural checker obligations that reject configured raw state aliases and raw transition bypasses while accepting nominal marker-state APIs with visible transition methods.

#### Scenario: Identity values use nominal wrappers [r[nominal-type-boundary-guidance-scenario-identity-values-use-nominal-wrappers]]

- GIVEN a public API accepts an account ID, job ID, token, unit-bearing value, or secret
- WHEN type-boundary guidance is applied
- THEN the API prefers a named wrapper over a raw primitive when confusion would be safety-relevant

#### Scenario: Structural records remain intentional [r[nominal-type-boundary-guidance-scenario-structural-records-remain-intentional]]

- GIVEN two data shapes are intentionally interchangeable plain data
- WHEN structural guidance is applied
- THEN a plain record or tuple remains acceptable with documented interchangeability

#### Scenario: Compliant nominal marker-state API passes [r[nominal-type-boundary-guidance-scenario-compliant-typestate-passes]]

- GIVEN a generated policy artifact declares type-state source scopes, required state marker names, and required transition method names
- WHEN `tigerstyle-standards check --policy-artifacts-dir` scans a compliant source root with nominal marker states and transition methods
- THEN the type-state structural policy emits no diagnostics for that compliant source root

#### Scenario: Raw protocol bypass fails structurally [r[nominal-type-boundary-guidance-scenario-raw-protocol-bypass-fails]]

- GIVEN a generated policy artifact declares raw state alias and raw transition bypass patterns
- WHEN `tigerstyle-standards check --policy-artifacts-dir` scans a declared source root containing one of those raw bypasses
- THEN the type-state structural policy emits an error diagnostic naming the configured bypass pattern and exits nonzero when hard policies fail

### Requirement: Primitive Alias Lint Staging [r[primitive-alias-lint-staging]]

The system MUST classify future primitive-alias or missing-newtype lint candidates as staged until fixtures prove that warnings identify safety-relevant ambiguity.

#### Scenario: Primitive lint needs evidence [r[primitive-alias-lint-staging-scenario-primitive-lint-needs-evidence]]

- GIVEN a future checker flags raw IDs or unit values
- WHEN hard-gate promotion is considered
- THEN positive and negative fixtures prove it avoids harmless local primitives

### Requirement: Deterministic Handler Test Guidance [r[deterministic-handler-test-guidance]]

The system MUST document deterministic handler testing for effectful boundaries, including fixed clocks, seeded randomness, recording stores, fake remotes, and configured failure handlers.

#### Scenario: Handler test is ambient-free [r[deterministic-handler-test-guidance-scenario-handler-test-is-ambient-free]]

- GIVEN a test exercises effect-boundary logic
- WHEN deterministic handler guidance is applied
- THEN the test uses injected handler state rather than live time, randomness, network, or filesystem

#### Scenario: Recording handler proves requested effects [r[deterministic-handler-test-guidance-scenario-recording-handler-proves-requested-effects]]

- GIVEN core code emits effect descriptions
- WHEN a recording handler test runs
- THEN the test can assert the exact ordered effects without performing real side effects

### Requirement: Invalid Path Handler Evidence [r[invalid-path-handler-evidence]]

The system MUST require handler tests to cover invalid paths and cleanup obligations where the effect contract includes failure behavior.

#### Scenario: Failure handler proves cleanup [r[invalid-path-handler-evidence-scenario-failure-handler-proves-cleanup]]

- GIVEN an effect handler can fail after partial progress
- WHEN invalid-path evidence is captured
- THEN the evidence shows the reported error and cleanup or no-mutation behavior

### Requirement: Policy Enforcement Mode Taxonomy [r[policy-enforcement-mode-taxonomy]]

The system MUST let each Tiger Style policy declare a current enforcement mode and target enforcement mode chosen from `lint`, `gate`, `structural`, `verified`, and `human-oracle`.

Each mode declaration MUST name the checker or mechanism responsible for enforcement, the escalation criteria for moving to a stronger mode, and the accepted fallback when a stronger mode is not yet possible.

#### Scenario: Lint-only policy remains explicit [r[policy-enforcement-mode-taxonomy.scenario.lint-only-explicit]]

- GIVEN a Tiger Style policy has useful signal but no reliable structural Rust mechanism
- WHEN the policy registry is validated
- THEN the policy remains valid only if it declares `current = "lint"` or `current = "human-oracle"`
- AND the entry names why structural or verified enforcement is not currently required

#### Scenario: Target mode stronger than current mode records escalation [r[policy-enforcement-mode-taxonomy.scenario.target-escalation-recorded]]

- GIVEN a policy currently gates diagnostics but should eventually become construction-enforced
- WHEN the policy registry is exported
- THEN the exported policy includes both current and target enforcement modes
- AND it includes the fixture or proof evidence required before promotion

### Requirement: Nickel Typed Policy Contracts [r[nickel-typed-policy-contracts]]

The system MUST define Tiger Style policy configuration in Nickel with typed, closed contracts for policies, enforcement modes, mechanisms, evidence, suppressions, proof rails, and reference seed manifests.

The Nickel source MUST be the authoring surface for policy intent and MUST export deterministic machine data consumed by Rust checkers; Rust checkers MUST NOT treat ad-hoc TOML, JSON, or CLI flags as the source of policy truth when a Nickel policy export exists.

#### Scenario: Closed Nickel policy contract rejects unknown fields [r[nickel-typed-policy-contracts.scenario.closed-contract-rejects-unknown-fields]]

- GIVEN a policy entry contains an undeclared field or misspelled enforcement mode
- WHEN Nickel policy validation runs
- THEN validation fails before source-code checkers run
- AND the diagnostic names the invalid policy entry and field

#### Scenario: Structural mode requires structural mechanism contract [r[nickel-typed-policy-contracts.scenario.structural-requires-mechanism]]

- GIVEN a policy declares `current = 'structural`
- WHEN Nickel cross-field validation evaluates the policy
- THEN the policy is valid only if `mechanism.kind` is one of `crate_boundary`, `capability_api`, `bounded_primitive`, `typestate`, or `seed_corpus`
- AND the required mechanism-specific contract fields are present

#### Scenario: Verified mode requires proof rail contract [r[nickel-typed-policy-contracts.scenario.verified-requires-proof-rail]]

- GIVEN a policy declares `current = 'verified`
- WHEN Nickel cross-field validation evaluates the policy
- THEN the policy is valid only if `mechanism.kind = 'proof_rail`
- AND the proof backend, command, source paths, expected signal, and trust-boundary accounting are present

#### Scenario: Deny severity requires deterministic checker and evidence [r[nickel-typed-policy-contracts.scenario.deny-requires-checker-evidence]]

- GIVEN a policy declares `severity = 'deny`
- WHEN Nickel cross-field validation evaluates the policy
- THEN the policy is valid only if a deterministic checker command and expected evidence signal are configured
- AND suppressions, when allowed, require policy ID, source section, owner, rationale, review trigger, and safer default metadata

### Requirement: Nickel Mechanism Contracts [r[nickel-mechanism-contracts]]

The system MUST define mechanism-specific Nickel contracts for crate boundaries, capability APIs, bounded primitives, type-state protocols, proof rails, and seed corpora.

Mechanism contracts MUST include the fields needed by Rust checkers to validate the corresponding structural or verified claim without inferring policy intent from prose.

For bounded primitive mechanisms, the exported contract MUST include approved bounded APIs or wrapper scopes, forbidden unbounded APIs, capacity-witness requirements, and backpressure or overflow behavior requirements.

For type-state mechanisms, Nickel contracts MUST support deterministic exported fields for source scopes, raw state alias patterns, raw transition patterns, required state marker names, and required transition method names.

#### Scenario: Bounded queue artifact exports checker obligations [r[nickel-mechanism-contracts.scenario.bounded-queue-artifact-obligations]]

- GIVEN `tigerstyle.embedded.bounded-queues` declares `mechanism.kind = 'bounded_primitive`
- WHEN generated policy artifacts are exported
- THEN the bounded queue policy artifact includes forbidden unbounded constructors, approved bounded constructors or wrapper scopes, required capacity-witness fields, and required backpressure or overflow behavior fields
- AND the artifact preserves `tigerstyle.embedded.bounded-queues` as the policy ID consumed by `tigerstyle-standards check --policy-artifacts-dir`

#### Scenario: Type-state obligations export deterministically [r[nickel-mechanism-contracts-scenario-type-state-obligations-export]]

- GIVEN the policy registry declares a `type-state` mechanism for nominal type boundaries
- WHEN generated policy artifacts are exported from the Nickel registry
- THEN the policy JSON and checker config include the configured type-state source scopes, raw bypass patterns, state marker names, and transition method names

### Requirement: Structural Mechanism Validation [r[structural-mechanism-validation]]

The system MUST validate claimed structural enforcement mechanisms before accepting a policy as structurally enforced.

Structural validation MUST cover at least no-std core crates, shell-only side effects, approved capability constructors, bounded primitive use, type-state protocol surfaces, and crate dependency direction.

For bounded queue policies, structural validation MUST reject configured unbounded queue constructors in selected source roots and MUST accept only approved bounded constructors or wrappers with explicit capacity and backpressure witnesses.

For effect capability policies, structural validation MUST reject configured ambient authority APIs in declared core source scopes while accepting shell-owned constructors and injected capability or effect-data APIs.

For type-state protocol policies, structural validation MUST reject configured raw state aliases and raw transition shortcuts in declared source scopes while accepting nominal marker-state wrappers with declared transition methods.

#### Scenario: Artifact-backed checker rejects configured unbounded constructors [r[structural-mechanism-validation.scenario.bounded-queue-rejects-unbounded-constructors]]

- GIVEN a generated policy artifact for `tigerstyle.embedded.bounded-queues` lists Tokio, Crossbeam, or std unbounded channel constructors as forbidden APIs
- WHEN `tigerstyle-standards check --policy-artifacts-dir <dir> --family embedded` scans a source root containing one configured unbounded constructor
- THEN the checker exits non-zero
- AND the diagnostic includes policy ID `tigerstyle.embedded.bounded-queues`, structural enforcement mode, bounded primitive mechanism, source path, line when available, and remediation to use an explicit bounded queue with backpressure

#### Scenario: Bounded construction with witness passes [r[structural-mechanism-validation.scenario.bounded-queue-witness-passes]]

- GIVEN source code constructs queues only through approved bounded APIs or approved wrapper scopes
- AND each construction has an explicit capacity constant, capacity argument, or configured wrapper witness plus backpressure or overflow behavior
- WHEN the artifact-backed embedded structural checker scans the source root
- THEN the checker reports zero bounded queue diagnostics
- AND policy evaluation metadata still records that `tigerstyle.embedded.bounded-queues` was evaluated as a structural bounded primitive policy

#### Scenario: Missing capacity witness remains a structural finding [r[structural-mechanism-validation.scenario.bounded-queue-missing-witness-fails]]

- GIVEN source code hides queue construction behind a wrapper or helper that does not expose a configured capacity witness or backpressure behavior
- WHEN the artifact-backed embedded structural checker scans the source root
- THEN the checker reports a structural finding for `tigerstyle.embedded.bounded-queues`
- AND the remediation names the missing capacity or backpressure witness rather than reporting a generic lint failure

#### Scenario: Downstream mutation fails the policy artifact consumer check [r[structural-mechanism-validation.scenario.bounded-queue-downstream-mutation-fails]]

- GIVEN Aspen pins a Tigerstyle flake exposing `lib.mkPolicyArtifactConsumerCheck` and checks selected embedded roots with `checks.x86_64-linux.tigerstyle-policy-artifacts`
- WHEN a scratch Aspen copy inserts exactly one configured unbounded queue constructor inside a checked root
- THEN the focused Aspen policy artifact check exits non-zero
- AND the saved transcript names `tigerstyle.embedded.bounded-queues`, the mutated file, and the single expected mutation diagnostic before the scratch mutation is removed or discarded

#### Scenario: Focused Tigerstyle checks pass [r[structural-mechanism-validation-scenario-focused-typestate-checks-pass]]

- GIVEN the type-state structural checker, fixtures, policy registry, and generated artifact export are changed together
- WHEN the focused Rust, Nickel/Nix, OpenSpec, docs, and whitespace checks run
- THEN every focused check exits successfully and the saved transcript names the commands used

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]

The system MUST require proof-rail evidence when a policy declares `current = "verified"` or targets verified enforcement for critical primitives.

Verified enforcement MUST name the proof backend, source/proof mapping, expected command, accepted proof artifact shape, and trust-boundary accounting. Verified enforcement MUST also provide artifact-backed checker diagnostics when declared local proof evidence is absent or does not contain the configured pass signal.

#### Scenario: Proof rail obligations are exported [r[verified-enforcement-rails.scenario.proof-rail-obligations-exported]]

- GIVEN a policy declares verified enforcement with proof rail metadata
- WHEN generated policy artifacts are exported
- THEN the generated check config summarizes proof rail backends, commands, source paths, proof paths, pass signals, and trust boundaries
- AND the artifact-backed checker can consume those proof rail fields without reading Nickel source directly

#### Scenario: Compliant proof rail receipt passes [r[verified-enforcement-rails.scenario.compliant-proof-rail-passes]]

- GIVEN checked source includes the declared source path and a declared proof receipt path
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the verification family
- THEN the checker exits zero when the proof receipt contains the configured pass signal
- AND the evaluated policy metadata classifies the failure type as `verified-proof-rail`

#### Scenario: Missing proof pass signal fails [r[verified-enforcement-rails.scenario.missing-proof-pass-signal-fails]]

- GIVEN checked source includes a proof receipt path without the configured pass signal
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the verification family
- THEN the checker exits nonzero with a verified-proof diagnostic naming the missing pass signal
- AND the remediation tells the developer to rerun the proof command and save a passing receipt

#### Scenario: Focused implementation checks pass [r[verified-enforcement-rails.scenario.focused-implementation-checks-pass]]

- GIVEN the proof-rail checker, artifacts, registry, fixtures, and OpenSpec package have been updated
- WHEN the focused Rust, Nickel/Nix, and OpenSpec validation commands run
- THEN every command exits successfully
- AND the saved transcript names the exact commands used

#### Scenario: Proof rail gate does not overclaim semantic proof execution [r[verified-enforcement-rails.scenario.no-semantic-proof-overclaim]]

- GIVEN the checker validates proof receipt shape and pass-signal presence
- WHEN reviewers inspect the design and evidence
- THEN the change states that `tigerstyle-standards check` does not execute arbitrary proof commands
- AND semantic proof execution remains the responsibility of the saved deterministic evidence command outside the pure checker

### Requirement: Reference Seed Corpus Vendoring [r[reference-seed-corpus-vendoring]]

The system MUST support optional vendoring of reference code, fixtures, examples, or proof seeds from `../vericoder-mvp/`, `../verified-logic/`, and `../ucan/` when those seeds help implement structural or verified Tiger Style enforcement.

Vendored seeds MUST include a manifest recording source repository, source commit, selected paths, BLAKE3 or SHA-256 hashes, license/provenance review, intended role, local acceptance command, and whether the seed is advisory, fixture-only, reusable library code, or proof-relevant.

#### Scenario: Vericoder seed is accepted as architecture or trajectory fixture [r[reference-seed-corpus-vendoring.scenario.vericoder-seed-fixture]]

- GIVEN a future implementation imports Vericoder MVP core/shell patterns, trajectories, or analyzer fixtures
- WHEN the seed manifest is validated
- THEN the seed is accepted only if it records the source commit, path list, hashes, and local Tigerstyle acceptance command
- AND the manifest marks the seed as advisory architecture/fixture evidence rather than proof of Tigerstyle correctness

#### Scenario: Verified-logic seed requires local proof boundary [r[reference-seed-corpus-vendoring.scenario.verified-logic-proof-boundary]]

- GIVEN a future implementation imports proof-relevant primitives from verified-logic
- WHEN the seed is used for a verified Tiger Style policy
- THEN Tigerstyle validation requires local proof or a recorded trust-boundary contract in this repository
- AND sibling repository proof status alone is not accepted as the final evidence artifact

#### Scenario: UCAN seed demonstrates core-shell vendoring shape [r[reference-seed-corpus-vendoring.scenario.ucan-core-shell-seed]]

- GIVEN a future implementation imports UCAN examples of a no-std core, std shell, or vendored verified-logic bridge
- WHEN structural validation runs against the seed fixture
- THEN the fixture proves the checker accepts the intended core/shell dependency direction and vendored-revision manifest shape
- AND a negative fixture proves ambient effects in the core still fail

### Requirement: Enforcement Mode Output and CI Behavior [r[enforcement-mode-output-ci-behavior]]

The system MUST expose enforcement-mode validation results through deterministic human output, machine output, and local-first CI or Nix gates.

Reports MUST distinguish lint findings, gate failures, structural-contract failures, verified-proof failures, accepted suppressions, and advisory/human-oracle reminders.

#### Scenario: Machine output classifies enforcement failure type [r[enforcement-mode-output-ci-behavior.scenario.machine-output-classifies-failure]]

- GIVEN a workspace violates one lint-only policy and one structural policy
- WHEN the checker emits machine-readable output
- THEN each finding includes policy ID, enforcement mode, failure kind, configured mechanism, source location when available, and remediation summary
- AND structural-contract failures are distinguishable from ordinary lint diagnostics

#### Scenario: Nix or CI gate fails on missing structural evidence [r[enforcement-mode-output-ci-behavior.scenario.ci-fails-missing-structural-evidence]]

- GIVEN a policy is configured as structurally enforced
- WHEN the local-first CI or Nix check runs without the required crate/API/fixture evidence
- THEN the check exits non-zero
- AND the diagnostic names the missing structural mechanism rather than reporting a generic lint failure

### Requirement: Generated Policy Artifacts [r[generated-policy-artifacts]]

The system MUST export deterministic generated artifacts from the validated Tiger Style policy registry for consumer checkers and CI review.

The artifact bundle MUST include `tigerstyle-policies.json`, `tigerstyle-check-config.toml`, and `tigerstyle-seed-manifests.json`.

#### Scenario: Artifact export writes named files [r[generated-policy-artifacts.scenario.named-files]]

- GIVEN a registry JSON exported from `standards/policies/registry.ncl`
- WHEN the standards artifact export command writes to an output directory
- THEN the output directory contains `tigerstyle-policies.json`
- AND the output directory contains `tigerstyle-check-config.toml`
- AND the output directory contains `tigerstyle-seed-manifests.json`

#### Scenario: Artifact-backed checker run [r[generated-policy-artifacts.scenario.checker-consumes-artifacts]]

- GIVEN a generated artifact directory
- WHEN `tigerstyle-standards check` runs with that directory and a structural fixture
- THEN the checker loads the policy registry from `tigerstyle-policies.json`
- AND the checker emits the same policy diagnostics and enforcement metadata as the registry-backed path

#### Scenario: Check config summarizes enforcement seams [r[generated-policy-artifacts.scenario.check-config-summary]]

- GIVEN generated policy artifacts
- WHEN the check config artifact is inspected
- THEN it lists checker IDs, policy families, hard policy IDs, structural policy IDs, and verified policy IDs in deterministic order

### Requirement: Artifact-backed effect capability mechanism obligations [r[artifact-backed-effect-capability-mechanism-obligations]]

The system MUST let the `tigerstyle.architecture.effect-capability-boundaries` policy export capability-api mechanism obligations through generated policy artifacts.

#### Scenario: Capability obligations are exported [r[artifact-backed-effect-capability-mechanism-obligations.scenario.exported]]

- GIVEN the Nickel registry declares a capability-api mechanism for effect capability boundaries
- WHEN generated policy artifacts are exported
- THEN the `tigerstyle.architecture.effect-capability-boundaries` artifact entry carries forbidden ambient APIs, core source scopes, shell constructor scopes, and approved capability names
- AND the artifact-backed checker can consume those fields without reading Nickel source directly

### Requirement: Core ambient authority structural rejection [r[core-ambient-authority-structural-rejection]]

The system MUST reject declared core source code that performs configured ambient authority operations instead of accepting injected data or narrow capability traits.

#### Scenario: Core ambient operation fails [r[core-ambient-authority-structural-rejection.scenario.core-ambient-fails]]

- GIVEN a Tigerstyle architecture fixture whose declared core scope calls a configured ambient authority API such as `std::fs`, `std::env`, `std::net`, `SystemTime::now`, or process spawning
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the architecture family
- THEN the report contains a structural diagnostic for `tigerstyle.architecture.effect-capability-boundaries`
- AND the diagnostic remediation tells the developer to move ambient constructors to the shell and inject data or named capabilities into core

### Requirement: Injected capability and shell construction acceptance [r[injected-capability-shell-construction-acceptance]]

The system MUST accept declared core source code that represents effects as data or named capability traits while allowing ambient constructors in declared shell construction scopes.

#### Scenario: Injected core capability passes [r[injected-capability-shell-construction-acceptance.scenario.injected-core-passes]]

- GIVEN a Tigerstyle architecture fixture whose core scope accepts a named capability trait or effect data instead of reading ambient state directly
- WHEN the artifact-backed architecture check runs
- THEN no `tigerstyle.architecture.effect-capability-boundaries` diagnostic is emitted for the core fixture

#### Scenario: Shell constructor scope passes [r[injected-capability-shell-construction-acceptance.scenario.shell-scope-passes]]

- GIVEN a Tigerstyle architecture fixture whose declared shell scope owns the configured ambient constructor
- WHEN the artifact-backed architecture check runs
- THEN no `tigerstyle.architecture.effect-capability-boundaries` diagnostic is emitted for that shell fixture
