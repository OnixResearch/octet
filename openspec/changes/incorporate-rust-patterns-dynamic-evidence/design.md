## Context

The source material has two different roles. Valgrind contributes a product/workflow model: specialized tools, clear bug classes, profiling evidence, CI/release timing, and explicit slowdown trade-offs. Rust Design Patterns contributes language-native structures that can encode Tiger Style obligations in types and modules. Tigerstyle should absorb those concepts only when they strengthen safety, performance evidence, explicitness, boundedness, recoverability, or developer comprehension.

## Goals / Non-Goals

**Goals:**
- Convert selected external concepts into Tigerstyle-owned standards guidance and policy candidates.
- Separate immediate documentation/checklist work from future lint implementation.
- Define dynamic evidence profiles for unsafe/FFI/performance/concurrency review without making every consumer install heavyweight profilers.

**Non-Goals:**
- Import the full generic design-pattern catalog.
- Add heavyweight profiler/sanitizer dependencies to every default check.
- Promote noisy lints before policy classification and fixtures exist.

## Decisions

### 1. Treat external references as classified source material

**Choice:** Add a source-incorporation rule that records each imported Valgrind or Rust-patterns concept with a Tiger Style rationale and classification.

**Rationale:** The references are useful, but generic patterns are not automatically Tiger Style. Classification prevents cargo-cult docs and premature hard gates.

**Alternative:** Copy all relevant pattern pages into docs. Rejected because it blurs ownership and imports patterns that may conflict with Tigerstyle simplicity.

**Implementation:** Update standards policy/docs with entries for tool-suite profiles and Rust idioms, each tagged hard/advisory/human-process-only.

**Covers:** Scenario: External concepts are classified before incorporation

### 2. Make dynamic evidence opt-in profiles, not default lints

**Choice:** Document Valgrind-style evidence profiles for memory/allocation, cache/hot-path, race/concurrency, unsafe/FFI, and release readiness.

**Rationale:** Dynamic tools can be high signal but are slow and platform-sensitive. They belong in named evidence profiles with acceptance criteria rather than unconditional local checks.

**Alternative:** Add Valgrind or sanitizer runs to default `nix flake check`. Rejected because default checks must stay local-first and reasonably fast.

**Implementation:** Add docs/templates naming when a profile applies, expected command shape, expected pass signal, and how to save evidence under OpenSpec changes.

**Covers:** Scenario: Dynamic profile guidance names run timing and acceptance signal; Scenario: Heavyweight tools remain optional until a profile is selected

### 3. Promote Rust idioms that encode Tiger Style contracts

**Choice:** Prioritize newtypes, RAII guards, contained unsafe modules, borrowed argument types, temporary mutability, clone-avoidance, recoverable consumed-argument errors, constrained builders, command/effects-as-data, FFI wrappers/errors, Deref restraint, and small-crate boundaries.

**Rationale:** These idioms make units, resources, ownership, boundaries, and recovery explicit, matching existing Tigerstyle principles.

**Alternative:** Add generic behavioural/creational/structural pattern docs. Rejected because many patterns do not improve Tigerstyle enforcement.

**Implementation:** Create a standards guide/checklist and policy roadmap rows; defer individual lints to child OpenSpecs with fixtures.

**Covers:** Scenario: Rust idiom guidance maps each imported pattern to Tiger Style outcomes; Scenario: Future lints require fixtures before hard gating

## Risks / Trade-offs

**Scope creep** → Keep the first slice documentation/policy-roadmap only; require separate child OpenSpecs for actual lints.

**Tool availability differences** → Dynamic profile docs must include optional/tool-specific language and one profile-selected acceptance signal, not a universal install requirement.

**Noisy static checks** → Pattern-derived lint candidates start advisory until positive and negative fixtures prove low false-positive behavior.
