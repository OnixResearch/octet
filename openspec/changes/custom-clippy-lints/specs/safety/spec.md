# Safety Lints — Delta Spec

## ADDED Requirements

### Requirement: Compound Assertion Detection

The lint MUST detect `assert!(expr1 && expr2)` and `assert!(expr1 || expr2)`
where the macro argument contains a top-level binary `&&` or `||` operator.

The lint MUST suggest splitting into separate `assert!()` calls.

The lint MUST NOT fire on `assert_eq!` or `assert_ne!` (those already provide
per-operand diagnostics).

The lint SHOULD NOT fire on `debug_assert!` compound expressions in test code.

#### Scenario: Simple compound

- GIVEN `assert!(a > 0 && b > 0)`
- WHEN the lint runs
- THEN it emits a warning suggesting `assert!(a > 0); assert!(b > 0);`

#### Scenario: Nested boolean in method call

- GIVEN `assert!(vec.iter().all(|x| x > 0 && x < 100))`
- WHEN the lint runs
- THEN it does NOT fire (the `&&` is inside a closure, not top-level)

#### Scenario: Conditional assertion (implication pattern)

- GIVEN `if is_leader { assert!(has_quorum); }`
- WHEN the lint runs
- THEN it does NOT fire (this is the approved Tiger Style pattern)

### Requirement: Assertion Density

The lint SHOULD warn on functions exceeding 20 lines that contain fewer
than 2 assertion macros (`assert!`, `assert_eq!`, `assert_ne!`,
`debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`).

The lint MUST NOT fire on functions annotated `#[test]`, `#[cfg(test)]`,
or inside `mod tests` blocks.

The lint MUST NOT fire on functions whose body is a single expression or
a pure delegation (`self.inner.method(args)`).

#### Scenario: Function with no assertions

- GIVEN a 30-line function with zero assertions
- WHEN the lint runs
- THEN it emits a suggestion to add assertions documenting invariants

#### Scenario: Short function

- GIVEN a 10-line function with zero assertions
- WHEN the lint runs
- THEN it does NOT fire (under the 20-line threshold)

### Requirement: Unbounded Loop Detection

The lint MUST warn on `loop { }` blocks that do not contain a visible
iteration counter, `break` condition tied to a counter, or bounded
channel receive.

The lint SHOULD warn on `while let Some(_) = iter.next()` without a
limit check in the body.

The lint MUST NOT fire on `loop` blocks containing `tokio::select!`
(event loops are a valid pattern when bounded by channel capacity).

#### Scenario: Bare retry loop

- GIVEN `loop { if try_connect() { break; } }`
- WHEN the lint runs
- THEN it warns about missing iteration bound

#### Scenario: Bounded retry

- GIVEN `for _ in 0..MAX_RETRIES { if try_connect() { break; } }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Ignored Result Detection

The lint MUST detect `let _ = expr` where `expr` has return type
`Result<_, _>`.

The lint MUST suggest using `let _: () = expr?;` or explicit error
handling.

The lint MUST NOT fire on `let _ = expr` where `expr` returns a
non-Result type (e.g., guard drops).

#### Scenario: Silenced fallible call

- GIVEN `let _ = file.sync_all();`
- WHEN the lint runs
- THEN it warns that a `Result` is being silently discarded
