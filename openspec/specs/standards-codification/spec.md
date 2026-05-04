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
