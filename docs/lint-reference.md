# Lint reference

This page lists the shipped tigerstyle lints, their default levels, and the
supported suppression and config paths.

For consumer setup, see `consumer-guide.md`.
For the landing-page naming map, see `../README.md`.

## Catalog summary

`tigerstyle` ships 35 lints across 5 domains.

| Domain | Group | Count |
| --- | --- | ---: |
| safety | `tigerstyle::safety` | 17 |
| naming | `tigerstyle::naming` | 5 |
| structure | `tigerstyle::structure` | 9 |
| purity | `tigerstyle::purity` | 3 |
| concurrency | `tigerstyle::concurrency` | 1 |

## Suppression and level controls

Source attrs and group attrs use canonical tool names:

```rust
#[allow(tigerstyle::unchecked_narrowing)]
#![deny(tigerstyle::naming)]
#![warn(tigerstyle::numeric_units)]
```

Keep config keys bare in `dylint.toml`:

```toml
[tigerstyle]
disabled_lints = ["no_unwrap"]

[tigerstyle.lint_levels]
numeric_units = "warn"
no_unwrap = "deny"
```

Rules to keep straight:

- source attrs and domain groups use `tigerstyle::...`
- `disabled_lints` and `[tigerstyle.lint_levels]` use bare lint names
- `disabled_lints` wins over `lint_levels`
- allow-by-default lints can be turned on with `#![warn(...)]` or a lint-level override
- standalone raw-rustc crates and UI fixtures that use tool-scoped attrs need:

```rust
#![feature(register_tool)]
#![register_tool(tigerstyle)]
```

## Config keys

| Key | Default | Used by | Notes |
| --- | --- | --- | --- |
| `function_length_max` | `70` | `function_length` | maximum non-blank, non-comment lines in a function body |
| `nesting_max_depth` | `3` | `nested_conditionals` | maximum `if` / `match` nesting depth |
| `assertion_density_min_lines` | `20` | `assertion_density` | minimum function size before assertion-density checks apply |
| `assertion_density_min_assertions` | `2` | `assertion_density` | minimum assertion count once the line threshold is crossed |
| `max_parameters` | `5` | `too_many_parameters` | maximum counted parameters before warning |
| `acronym_allowlist` | `[]` | `acronym_style` | case-sensitive allowlist such as `IO` or `ID` |
| `disabled_lints` | `[]` | all lints | bare lint names only; disabled lints do not register |
| `[tigerstyle.lint_levels]` | `{}` | all lints | bare lint names mapped to `allow`, `warn`, or `deny` |

Unknown lint names and invalid level values warn and are ignored.

## Safety lints (`tigerstyle::safety`)

| Lint | Default | Summary |
| --- | --- | --- |
| `compound_assertion` | `warn` | split `assert!(a && b)` into separate assertions |
| `ignored_result` | `warn` | avoid silent `let _ = expr` drops for fallible results |
| `no_unwrap` | `warn` | ban `.unwrap()` and `.expect()` in production code |
| `no_panic` | `warn` | ban `panic!()`, `unimplemented!()`, and `unreachable!()` in production code |
| `no_todo` | `warn` | ban `todo!()` in production code |
| `unjustified_no_todo_allow` | `warn` | require a local `//` justification for `allow(no_todo)` / `allow(tigerstyle::no_todo)` |
| `no_recursion` | `warn` | flag direct self-recursion |
| `unchecked_narrowing` | `warn` | flag `as u32`-style narrowing without a checked bound |
| `unchecked_division` | `warn` | require a zero guard before dividing by a variable |
| `unbounded_loop` | `warn` | flag `loop {}` without an explicit bound or exit condition |
| `catch_all_on_enum` | `warn` | flag `_` match arms that hide future enum variants |
| `explicit_defaults` | `warn` | avoid foreign `Default` shorthands such as `..Default::default()` |
| `unbounded_channel` | `warn` | flag known unbounded channel constructors |
| `unbounded_collection_growth` | `warn` | flag loop-driven collection growth without reservation or a local bound |
| `assertion_density` | `allow` | flag large functions with too few assertions |
| `raw_arithmetic_overflow` | `allow` | flag unchecked `+`, `-`, or `*` on unsigned integers |
| `sentinel_fallback` | `warn` | flag `<int>::MAX` / `<int>::MIN` sentinel fallbacks instead of explicit branching or tagged fallback text |

## Naming lints (`tigerstyle::naming`)

| Lint | Default | Summary |
| --- | --- | --- |
| `acronym_style` | `warn` | prefer `RpcHandler` over `RPCHandler` |
| `bool_naming` | `warn` | require predicate-style boolean names like `is_ready` |
| `negated_predicate` | `warn` | avoid names like `is_not_ready` or `no_capacity` |
| `numeric_units` | `allow` | require unit suffixes such as `_ms`, `_secs`, or `_bytes` |
| `float_for_currency` | `allow` | avoid `f64` in price, cost, or budget fields |

## Structure lints (`tigerstyle::structure`)

| Lint | Default | Summary |
| --- | --- | --- |
| `function_length` | `warn` | flag functions longer than the configured line budget |
| `nested_conditionals` | `warn` | flag deep `if` / `match` nesting |
| `platform_dependent_cast` | `warn` | flag casts like `as usize` from fixed-width integers |
| `usize_in_public_api` | `warn` | avoid `usize` / `isize` at inherent public API boundaries |
| `borrowed_argument_types` | `warn` | prefer `&str`, `&[T]`, and `&Path` over borrowed owned containers |
| `too_many_parameters` | `warn` | flag signatures that exceed the configured parameter limit |
| `compound_condition` | `allow` | flag conditions with 3 or more boolean operators |
| `unjustified_allow` | `allow` | require local justification for `#[allow(clippy::...)]` |
| `ambiguous_params` | `allow` | flag consecutive same-type primitive parameters |

## Purity lints (`tigerstyle::purity`)

| Lint | Default | Summary |
| --- | --- | --- |
| `ambient_clock` | `warn` | keep wall-clock reads at explicit shell boundaries |
| `verified_purity` | `deny` | ban async, IO, and wall-clock usage in `verified/` modules |
| `contradictory_time` | `warn` | avoid taking time as input and then reading the clock again |

## Concurrency lints (`tigerstyle::concurrency`)

| Lint | Default | Summary |
| --- | --- | --- |
| `multi_lock_ordering` | `allow` | require an ordering comment when a struct owns multiple locks |

## Related docs

- `consumer-guide.md` — setup and rollout entry points
- `downstream-rollout.md` — rollout profiles and Aspen-backed evidence
- `../templates/dylint.toml` — copyable consumer template with rollout comments
