## Context

The high-noise lint promotion playbook names five families that stay
inventory-first because of false-positive volume. Aspen phase-2 widening
confirmed the noise: `aspen-coordination` alone had 21 `ambient_clock`,
4 `no_panic`, and additional `no_unwrap` findings. Precision improvements
should reduce noise enough that these families become promotable.

## Goals / Non-Goals

**Goals:** Reduce false positive rate in the five named families without
weakening detection of real bugs.

**Non-Goals:** Eliminating all false positives. Some findings are
genuinely noisy because the underlying code pattern is debatable — those
belong in the narrow-allow workflow, not in lint suppression.

## Decisions

### 1. Test-context suppression for `no_unwrap` and `no_panic`

**Choice:** Both lints skip `#[test]` functions and `#[cfg(test)]` modules.

**Rationale:** Test code uses `.unwrap()` and `panic!` legitimately for
test assertions and setup. These are already excluded by spec for other
lints like `assertion_density`.

**Implementation:** Check `cx.tcx.hir_attrs(hir_id)` for `test` or
`cfg(test)` attributes. Walk parent items for `mod tests` detection.

### 2. Compile-time-known unwrap suppression

**Choice:** `no_unwrap` skips `.unwrap()` on expressions that are
provably `Some`/`Ok` from literal construction (e.g., string `.parse()`
on a literal, `Some(x).unwrap()`).

**Rationale:** These are idiomatic Rust for const initialization and
zero-cost at runtime. Flagging them adds noise without catching bugs.

**Implementation:** Check if the receiver of `.unwrap()` is a call to
`parse()` on a string literal, `Some(...)`, or `Ok(...)`.

### 3. Drop-impl suppression for `ignored_result`

**Choice:** Skip `let _ = expr` inside `Drop::drop` implementations.

**Rationale:** `Drop::drop` cannot return `Result`. Ignoring errors
during cleanup is the only option. The error would have to be logged
or swallowed regardless.

**Implementation:** Check if the enclosing function is a `Drop::drop`
impl method.

### 4. Exhaustive-match unreachable suppression for `no_panic`

**Choice:** Skip `unreachable!()` in match arms that follow exhaustive
variant listing.

**Rationale:** After listing all enum variants explicitly, a trailing
`_ => unreachable!()` is defensive programming, not a liveness hazard.

**Implementation:** Check if the match arm is a wildcard following
variant-listing arms that cover all known variants.

**Alternative:** Only suppress when the compiler can prove exhaustiveness
— rejected because rustc's exhaustiveness check happens before our lint
pass and we'd need to duplicate that logic.

### 5. Broader bounds-check recognition for `unchecked_narrowing`

**Choice:** Recognize `x.min(T::MAX as u64) as T` and
`T::try_from(x).unwrap_or(T::MAX)` as bounded patterns.

**Rationale:** These are common bounded-narrowing idioms that don't need
a preceding `assert!`.

**Implementation:** Pattern-match on the expression structure surrounding
the `as` cast.

## Risks / Trade-offs

**[Over-suppression]** → Each new exception is a potential gap. Mitigation:
every suppression rule gets a UI test fixture proving both the suppression
and the continued detection of the real-bug case.

**[Downstream validation]** → Must re-run against Aspen to confirm noise
reduction without losing real findings. Mitigation: save before/after
finding counts as evidence.
