# Rust idiom guidance

Slotcar imports Rust idioms only when they make safety, explicitness, boundedness, recoverability, performance evidence, or developer comprehension stronger. Generic pattern names are not enough; every imported pattern must state the Tiger Style outcome it protects.

## Immediate guidance

| Concept | Tiger Style outcome | First enforcement tier |
| --- | --- | --- |
| Newtype | Units, IDs, capabilities, and secrets stop being interchangeable primitives | Advisory until a domain-specific hard gate has fixtures |
| RAII guard | Resource access is mediated by a type that releases or rolls back on scope exit | Human checklist; lint only for narrow known guard shapes |
| Contained unsafe module | Unsafe invariants stay in the smallest auditable boundary with a safe outer API | Advisory/hard depending on configured module allowlist and fixtures |
| Borrowed argument type | APIs accept `&str`, `&[T]`, `Path`, or other borrowed forms when ownership is unnecessary | Future lint candidate with positive and negative fixtures |
| Temporary mutability | Mutable setup ends before read-only use starts | Advisory; avoid hard gating without precise lifetime/use analysis |
| Clone-avoidance | `mem::take`, `mem::replace`, `Option::take`, and narrower borrows replace clone-to-appease-borrow-checker patterns | Advisory until local false-positive rate is known |
| Recoverable consumed argument errors | Fallible APIs that consume expensive/stateful input return it in the error for retry/recovery | Human API review; future lint only for named API families |
| Constrained builder | Many options or same-type parameters get named setters, explicit required fields, and `build()` invariant checks | Already related to parameter/default lints; docs first |
| Command/effects-as-data | Core returns commands/effects, shell executes them, rollback/replay is explicit | Architecture guidance and OpenSpec review |
| FFI wrappers and errors | Foreign APIs see stable handles/codes while Rust keeps lifetimes and rich errors internally | Human checklist plus optional dynamic evidence profile |
| Restrained `Deref` | `Deref` is for smart-pointer/guard ergonomics, not inheritance-like API smuggling | Advisory `deref_polymorphism` lint plus fixtures |
| Small crates | Crates enforce real boundaries such as core/shell, unsafe, FFI, policy, or verifier seams | Architecture guidance; hard only for declared repo roles |

## Review prompts

- What primitive, unit, state, or capability is currently unnamed?
- What invalid operation could become a compile error through a newtype, marker type, sealed trait, or RAII guard?
- What ownership or lifetime rule is currently enforced by convention instead of a safe wrapper?
- Which `clone()` calls are compensating for ownership structure rather than expressing a real copy requirement?
- Does a builder hide safety-sensitive defaults, or does `build()` validate the resulting object?
- Does FFI expose lifetimes or iterator state that the foreign caller can invalidate?
- Is `Deref` making an API look smaller while actually exposing too much behavior?

## Lint promotion rule

Pattern-derived lints stay advisory until they have:

1. A policy entry with source link and Tiger Style rationale.
2. At least one compliant fixture and one noncompliant fixture.
3. A documented false-positive boundary.
4. A staged rollout profile that starts with inventory-only or warning-level reporting.

## Non-goals

- Do not import a pattern because it is common in another language.
- Do not replace clear branches and assertions with clever combinators in safety-critical code.
- Do not hard-gate broad style preferences without fixture-backed precision.
