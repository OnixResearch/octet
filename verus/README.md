# Verus Specifications

**Status: experimental.** This directory is a placeholder for future
[Verus](https://verus-lang.github.io/verus/guide/) formal verification specs.
No verified properties are enforced by CI yet.

The `verified_purity` lint (see `docs/lint-reference.md`) bans async, IO, and
wall-clock usage in `verified/` modules, but Verus proof obligations are not
yet wired into the build.

## Contents

- `example_spec.rs` — skeleton showing `requires`/`ensures` clause structure

## Running verification

```bash
verus verus/example_spec.rs
```

## Relationship to Tiger Style

Tiger Style lints enforce coding discipline at the Rust compiler level. Verus
goes further by machine-checking preconditions and postconditions against
function bodies. The two are complementary: lints catch structural patterns,
Verus proves functional correctness for selected properties.

See the global `tigerstyle` skill for guidance on when to use combined vs split
spec patterns.
