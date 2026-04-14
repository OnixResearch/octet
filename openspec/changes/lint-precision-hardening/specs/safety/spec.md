# Safety Lint Precision Hardening — Delta Spec

## ADDED Requirements

### Requirement: No Unwrap In Production Code

The lint MUST detect `.unwrap()` and `.expect()` calls on `Option` and
`Result` in production code.

The lint MUST NOT fire in `#[test]` functions.

The lint MUST NOT fire in items or modules guarded by `#[cfg(test)]` or an
inner `#![cfg(test)]` that makes the enclosing item test-only.

The lint MUST NOT fire on compile-time-known infallible constructions such as
`Some(value).unwrap()`, `Ok(value).unwrap()`, or literal-string
`"127.0.0.1".parse().unwrap()`.

The lint MUST NOT fire when the unwrap is dominated by a same-branch
`.is_some()` or `.is_ok()` guard.

#### Scenario: Test helper unwrap

- GIVEN a `#[test]` function that calls `value.unwrap()`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Cfg-test module unwrap

- GIVEN a helper inside `#[cfg(test)] mod tests { ... }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Not-test cfg still warns

- GIVEN a helper inside `#[cfg(not(test))] mod prod_only { ... }`
- WHEN the lint runs
- THEN it still warns on `value.unwrap()`

#### Scenario: Compile-time-known unwrap

- GIVEN `let v = Some(1_u32).unwrap();`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Guarded unwrap

- GIVEN `if value.is_some() { value.unwrap(); }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Panic Macro In Production Code

The lint MUST detect `panic!()`, `todo!()`, `unimplemented!()`, and
`unreachable!()` in production code.

The lint MUST NOT fire in `#[test]` functions.

The lint MUST NOT fire in items or modules guarded by `#[cfg(test)]` or an
inner `#![cfg(test)]` that makes the enclosing item test-only.

The lint MUST NOT fire on `unreachable!()` in a trailing wildcard arm when the
preceding enum match arms already list the concrete variants exhaustively.

The lint MUST NOT fire on `todo!()` when the site is explicitly allowed with
`#[allow(tigerstyle::no_panic)]` and an adjacent justification comment.

#### Scenario: Test-only panic

- GIVEN a `#[test]` helper that calls `panic!("boom")`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Cfg-test module panic

- GIVEN a helper inside `#[cfg(test)] mod tests { ... }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Not-test cfg still warns

- GIVEN a helper inside `#[cfg(not(test))] mod prod_only { panic!(...) }`
- WHEN the lint runs
- THEN it still warns

#### Scenario: Exhaustive wildcard unreachable

- GIVEN an enum match whose explicit arms already cover every variant and whose
  trailing wildcard uses `unreachable!()`
- WHEN the lint runs
- THEN it does NOT fire

## MODIFIED Requirements

### Requirement: Ignored Result Detection

The lint MUST detect `let _ = expr` where `expr` has return type
`Result<_, _>`.

The lint MUST suggest using `let _: () = expr?;` or explicit error
handling.

The lint MUST NOT fire on `let _ = expr` where `expr` returns a
non-Result type (e.g., guard drops).

The lint MUST NOT fire inside `Drop::drop` implementations.

The lint SHOULD NOT fire on known fire-and-forget shutdown send paths where the
receiver may already be dropped.

#### Scenario: Drop cleanup swallow

- GIVEN `impl Drop for Cleaner { fn drop(&mut self) { let _ = self.tx.send(msg); } }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Unchecked Narrowing Cast

The lint MUST detect `as u32`, `as u16`, `as u8`, `as i32`, `as i16`,
`as i8` casts from a wider integer type without a prior bounds check
on the source value.

A bounds check is any of: `if value <= TYPE::MAX`, a preceding
`assert!(value <= ...)`, wrapping in `TryFrom`/`try_into()`, or calling
a checked narrowing function.

The lint MUST suggest using `u32::try_from(value)` or a checked
narrowing function like `verified_logic::conversion::narrow_u32`.

The lint MUST NOT fire on casts where the source type is the same width
or narrower than the target (e.g., `u16 as u32`).

The lint MUST NOT fire inside `#[allow(clippy::cast_possible_truncation)]`
blocks (explicit opt-out).

The lint MUST NOT fire on bounded forms such as `value.min(u32::MAX as u64) as u32`.

The lint MUST NOT fire on `u32::try_from(value).unwrap_or(u32::MAX)`.

The lint MUST treat a same-block `assert!(value <= u32::MAX as u64)` as a valid
bounds check even when it is not the immediately preceding statement.

#### Scenario: Min-bounded narrowing

- GIVEN `let id = value.min(u32::MAX as u64) as u32;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Try-from fallback narrowing

- GIVEN `let id = u32::try_from(value).unwrap_or(u32::MAX);`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Unbounded Loop Detection

The lint MUST warn on `loop { }` blocks that do not contain a visible
iteration counter, `break` condition tied to a counter, or bounded
channel receive.

The lint SHOULD warn on `while let Some(_) = iter.next()` without a
limit check in the body.

The lint MUST NOT fire on `loop` blocks containing `tokio::select!`
(event loops are a valid pattern when bounded by channel capacity).

The lint MUST NOT fire on `while let Some(_) = rx.recv().await` when `rx`
is a bounded receiver type.

#### Scenario: Bounded receiver loop

- GIVEN `while let Some(msg) = rx.recv().await { handle(msg); }`
- WHEN `rx` is a bounded receiver type
- THEN the lint does NOT fire
