## Why

`missing_const_fn` currently treats a comparison or arithmetic token as sufficient evidence of const eligibility. The HWR component identifier and versioned-interface predicates use non-const string iteration, parsing, and closures but receive an advisory. The canonical purity-effect contract explicitly excludes non-const calls.

## What Changes

- Retain the existing source guardrails, then require a conservative HIR body subset of primitive-valued expressions, without calls, method calls, closures, overloaded operations, or unsupported constructs.
- Limit reporting to free functions; do not recommend const for trait methods.
- Extend the existing `missing_const_fn` fixture with clean non-const string predicates, arbitrary calls, overloaded operators, and trait methods, retaining the arithmetic positive fixture.
- Document this precision boundary. Of the eight HWR findings, two const findings are defects and six remain governed by existing contracts (four naming, one construction, one density).

## Impact

- Specs: `purity-effect-lints` (conservative eligibility requirement clarified).
- Files: `src/purity/effect_lints.rs`, existing UI fixture, lint reference, changelog, native change artifacts.
- No consumer API edits, policy suppression, dependency changes, remote mutation, publication, or visibility change.

## Non-Goals

Do not prove general const eligibility or adopt rustc's full const checker. Do not exempt BLAKE3 by spelling or treat arbitrary external constructors as pure. The construction lint currently has no input carrying reviewed project-specific provider facts; importing Octet's self-policy into HWR would be unsound. Pure-provider policy integration requires an explicit design decision and is not silently added here. Naming and density policy remain intact. Octet's unresolved standards-prose publication hold remains in force.
