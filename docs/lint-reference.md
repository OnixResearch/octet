# Lint reference

This page lists the shipped Octet lints, their default levels, and the
supported suppression and config paths.

For consumer setup, see `consumer-guide.md`.
For checked-contract ladder context, see `trust-boundary-evidence.md#checked-contract-ladder`.
For the landing-page naming map, see `../README.md`.

## Catalog summary

`octet` ships 53 lints across 6 domains.

| Domain | Group | Count |
| --- | --- | ---: |
| safety | `tigerstyle::safety` | 23 |
| naming | `tigerstyle::naming` | 9 |
| performance | `tigerstyle::performance` | 1 |
| structure | `tigerstyle::structure` | 16 |
| purity | `tigerstyle::purity` | 3 |
| concurrency | `tigerstyle::concurrency` | 1 |

## Suppression and level controls

Source attrs and group attrs use the current live tool namespace:

```rust
#[allow(tigerstyle::unchecked_narrowing)]
#![deny(tigerstyle::naming)]
#![warn(tigerstyle::numeric_units)]
```

Keep config keys bare in `dylint.toml`:

```toml
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
numeric_units = "warn"
no_unwrap = "deny"
```

Rules to keep straight:

- source attrs and domain groups currently use `tigerstyle::...` until the namespace migration lands
- `disabled_lints` and `[octet.lint_levels]` use bare lint names
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
| `trait_method_max` | `7` | `fat_trait` | maximum trait methods before warning |
| `impl_blocks_max` | `4` | `type_accumulation` | maximum inherent impl blocks per type before advisory |
| `module_pub_items_max` | `12` | `type_accumulation` | maximum public items per module before advisory |
| `core_source_scopes` | `[]` | `concrete_construction_in_core` | source path scopes where concrete construction is checked |
| `sealed_enum_marker` | `"octet::sealed_enum"` | `fragile_exhaustive_enum_match` | attribute marker that suppresses fragile local exhaustive enum-match diagnostics |
| `acronym_allowlist` | `[]` | `acronym_style` | case-sensitive allowlist such as `IO` or `ID` |
| `disabled_lints` | `[]` | all lints | bare lint names only; disabled lints do not register |
| `[octet.lint_levels]` | `{}` | all lints | bare lint names mapped to `allow`, `warn`, or `deny` |

Unknown lint names and invalid level values warn and are ignored.

## Safety lints (`tigerstyle::safety`)

| Lint | Default | Summary |
| --- | --- | --- |
| `compound_assertion` | `warn` | split `assert!(a && b)` into separate assertions |
| `ignored_result` | `warn` | avoid silent `let _ = expr` drops for fallible results |
| `drop_must_use` | `warn` | handle must-use values explicitly instead of hiding them behind `drop(...)` |
| `fallible_int_fallback` | `warn` | propagate integer conversion failures instead of replacing them with sentinels or defaults |
| `no_unwrap` | `warn` | ban `.unwrap()` and `.expect()` in production code |
| `no_panic` | `warn` | ban `panic!()`, `unimplemented!()`, and `unreachable!()` in production code |
| `no_todo` | `warn` | ban `todo!()` in production code |
| `unjustified_no_todo_allow` | `warn` | require a local `//` justification for `allow(no_todo)` / `allow(tigerstyle::no_todo)` |
| `no_recursion` | `warn` | flag direct self-recursion |
| `unchecked_narrowing` | `warn` | flag `as u32`-style narrowing without a checked bound |
| `unchecked_division` | `warn` | require a zero guard before dividing by a variable |
| `unbounded_loop` | `warn` | flag `loop {}` without an explicit bound or exit condition |
| `catch_all_on_enum` | `warn` | flag `_` match arms that hide future enum variants |
| `foreign_enum_catch_all` | `allow` | flag `_` match arms over enums defined in another crate |
| `fragile_exhaustive_enum_match` | `allow` | flag exhaustive matches over same-crate enums without a fallback |
| `explicit_defaults` | `warn` | avoid foreign `Default` shorthands such as `..Default::default()` |
| `unbounded_channel` | `warn` | flag known unbounded channel constructors |
| `unbounded_collection_growth` | `warn` | flag loop-driven collection growth without reservation or a local bound |
| `assertion_density` | `allow` | flag large functions with too few assertions |
| `raw_arithmetic_overflow` | `allow` | flag unchecked `+`, `-`, or `*` on unsigned integers |
| `sentinel_fallback` | `warn` | flag `<int>::MAX` / `<int>::MIN` sentinel fallbacks instead of explicit branching or tagged fallback text |
| `public_unsafe_api` | `warn` | flag public Rust `unsafe fn` APIs that leak unsafe obligations to callers; prefer a safe wrapper or document the exception with an `UNSAFE-TRUST-BOUNDARY:` record |
| `defaulting_trait_method` | `allow` | flag trait methods with default bodies that can weaken substitution contracts |

## Naming lints (`tigerstyle::naming`)

| Lint | Default | Summary |
| --- | --- | --- |
| `acronym_style` | `warn` | prefer `RpcHandler` over `RPCHandler` |
| `bool_naming` | `warn` | require predicate-style boolean names like `is_ready` |
| `negated_predicate` | `warn` | avoid names like `is_not_ready` or `no_capacity` |
| `numeric_units` | `allow` | require unit suffixes such as `_ms`, `_secs`, or `_bytes` |
| `underscore_in_module_filename` | `warn` | prefer responsibility-based module directories over compound filenames |
| `non_trait_imports` | `warn` | keep concrete owner types qualified at use sites; import traits only for method resolution |
| `renamed_imports` | `warn` | avoid `use ... as ...` aliases that hide searchable item names |
| `path_segment_repetition` | `warn` | avoid repeating crate or module path words in item names |
| `float_for_currency` | `allow` | avoid `f64` in price, cost, or budget fields |

## Performance lints (`tigerstyle::performance`)

| Lint | Default | Summary |
| --- | --- | --- |
| `clone_then_borrow` | `warn` | flag cloning a value only to immediately take a shared borrow |

## Structure lints (`tigerstyle::structure`)

| Lint | Default | Summary |
| --- | --- | --- |
| `function_length` | `warn` | flag functions longer than the configured line budget |
| `magic_number` | `warn` | require named constants or config fields for policy thresholds and domain values |
| `excessive_file_length` | `warn` | split oversized source files along module or responsibility boundaries |
| `module_file_count` | `warn` | split crowded module directories into smaller responsibility-based submodules |
| `nested_conditionals` | `warn` | flag deep `if` / `match` nesting |
| `platform_dependent_cast` | `warn` | flag casts like `as usize` from fixed-width integers |
| `usize_in_public_api` | `warn` | avoid `usize` / `isize` at inherent public API boundaries |
| `borrowed_argument_types` | `warn` | prefer `&str`, `&[T]`, and `&Path` over borrowed owned containers |
| `deref_polymorphism` | `warn` | flag public `Deref` impls to nominal target types that hide API boundaries |
| `too_many_parameters` | `warn` | flag signatures that exceed the configured parameter limit |
| `fat_trait` | `warn` | flag traits exceeding the configured method budget |
| `concrete_construction_in_core` | `warn` | flag concrete construction calls in declared core source scopes |
| `type_accumulation` | `allow` | flag excessive inherent impl blocks or module public surface |
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
- `trust-boundary-evidence.md#checked-contract-ladder` — how lint findings become checked-contract ladder stages without becoming formal proof
- `rust-idiom-guidance.md#unsafe-trusted-boundary-records` — safe-wrapper and documented-exception paths for `public_unsafe_api`, FFI, raw-pointer, and lifetime-erasing boundaries
- `downstream-rollout.md` — rollout profiles and Aspen-backed evidence
- `../templates/dylint.toml` — copyable consumer template with rollout comments
