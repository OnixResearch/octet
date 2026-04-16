# tigerstyle

Custom [dylint](https://github.com/trailofbits/dylint) lints enforcing
[Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
coding principles for Rust.

32 lints across 5 domains: safety, naming, structure, purity, concurrency.

## Lint Catalog

### Safety (15 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `compound_assertion` | **warn** | `assert!(a && b)` should be split into separate assertions |
| `ignored_result` | **warn** | `let _ = expr` silently discards a `Result` |
| `no_unwrap` | **warn** | `.unwrap()`/`.expect()` in production code |
| `no_panic` | **warn** | `panic!()`/`todo!()`/`unimplemented!()` in production code |
| `unjustified_no_panic_allow` | **warn** | Invalid `#[allow(tigerstyle::no_panic)]` escape without adjacent `//` justification or around non-`todo!()` panic macros |
| `no_recursion` | **warn** | Function calls itself recursively |
| `unchecked_narrowing` | **warn** | `as u32` from wider type without bounds check |
| `unchecked_division` | **warn** | `a / b` where `b` is a variable without zero guard |
| `unbounded_loop` | **warn** | `loop {}` without visible iteration bound |
| `catch_all_on_enum` | **warn** | `_ => false` catch-all on enum match hides new variants |
| `explicit_defaults` | **warn** | Foreign `Default` use, including shorthand like `..Default::default()` |
| `unbounded_channel` | **warn** | Known unbounded channel constructors from std, crossbeam, or Tokio |
| `unbounded_collection_growth` | **warn** | Loop grows a local collection without prior reservation or a collection-local bound |
| `assertion_density` | allow | 20+ line function with fewer than 2 assertions |
| `raw_arithmetic_overflow` | allow | Unprotected `+`/`-`/`*` on unsigned integers |

### Naming (5 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `acronym_style` | **warn** | `RPCHandler` should be `RpcHandler` |
| `bool_naming` | **warn** | Boolean binding without `is_`/`has_`/`should_`/`can_` prefix |
| `negated_predicate` | **warn** | Bool APIs or locals named with prefixes like `is_not_` or `no_` |
| `numeric_units` | allow | Bare `timeout` without unit suffix (`_ms`, `_secs`) |
| `float_for_currency` | allow | `f64` in cost/price/budget struct fields |

### Structure (8 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `function_length` | **warn** | Function body exceeds 70 non-blank, non-comment lines |
| `nested_conditionals` | **warn** | `if`/`match` nesting exceeds 3 levels |
| `platform_dependent_cast` | **warn** | `as usize` from fixed-width integer is platform-dependent |
| `usize_in_public_api` | **warn** | Inherent public APIs should avoid `usize`/`isize` at boundaries |
| `too_many_parameters` | **warn** | Function signature exceeds the configured parameter limit |
| `compound_condition` | allow | `if` condition with 3+ boolean operators |
| `unjustified_allow` | allow | `#[allow(clippy::...)]` without justification comment |
| `ambiguous_params` | allow | 2+ consecutive same-type primitive parameters |

### Purity (3 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `ambient_clock` | **warn** | Direct wall-clock reads should stay at explicit shell boundaries |
| `verified_purity` | **deny** | async/IO/system time in `verified/` modules |
| `contradictory_time` | **warn** | Function takes time param but also calls `Instant::now()` |

### Concurrency (1 lint)

| Lint | Default | Description |
|------|---------|-------------|
| `multi_lock_ordering` | allow | Struct with 2+ Mutex/RwLock fields without ordering comment |

## Usage

### Prerequisites

- Nightly Rust toolchain with `rustc-dev` and `llvm-tools-preview` components
- the first-class `cargo-tigerstyle` runner, either through `nix run` or a
  locally available `cargo-tigerstyle` binary

### Running lints

Keep consumer-owned lint tuning in `dylint.toml`, then add runner policy in the
workspace `Cargo.toml`:

```toml
[workspace.metadata.tigerstyle]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

Run the first-class engine:

```bash
nix run path:/path/to/tigerstyle#cargo-tigerstyle -- check
# or, if the binary is already installed/on PATH:
cargo tigerstyle check
```

### Canonical source attrs and groups

Source-level lint attrs and group attrs use canonical tool names such as
`#[allow(tigerstyle::unchecked_narrowing)]` and `#![deny(tigerstyle::naming)]`.

Keep `dylint.toml` keys in bare form such as `no_unwrap`, not
`tigerstyle::no_unwrap`.

Standalone raw-rustc crates and UI fixtures that use tool-scoped tigerstyle
attrs need the supported nightly toolchain plus:

```rust
#![feature(register_tool)]
#![register_tool(tigerstyle)]
```

### Suppressing a lint

```rust
// Verified casts: Verus proves every narrowing in this module.
#[allow(tigerstyle::unchecked_narrowing)]
fn narrow(x: u64) -> u32 {
    x as u32
}
```

### Enabling an allow-by-default lint

```rust
#![warn(tigerstyle::numeric_units)]
#![warn(tigerstyle::assertion_density)]
```

### Per-project configuration

Place a `dylint.toml` in your workspace root to tune thresholds.
Keep `disabled_lints` and `[tigerstyle.lint_levels]` keys in bare lint-name
form:

```toml
[tigerstyle]
function_length_max = 100          # default: 70
nesting_max_depth = 4              # default: 3
assertion_density_min_lines = 30   # default: 20
assertion_density_min_assertions = 3  # default: 2
acronym_allowlist = ["IO", "ID"]   # default: []
```

All fields are optional — missing fields use the defaults shown above.

For a reusable consumer template, see `templates/dylint.toml`.

### Downstream rollout kit

Use these repo-local guides when onboarding a consumer workspace:

- `docs/downstream-rollout.md` — starter profiles, Aspen's phase-2 widening path, current Aspen classification for the six pending new families, `ambient_clock` boundary guidance, and the high-noise lint promotion playbook
- `templates/dylint.toml` — consumer-owned threshold, allowlist, and per-lint rollout template
- `openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/replay-aspen-new-lints.sh` — exact replay path for the saved Aspen new-family classification evidence

The Aspen phase-1 starter reference is archived under
`openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`.

## Development

```bash
nix develop                                   # enter devshell
cargo check                                   # verify compilation
cargo test                                    # run unit tests
cargo test --test ui -- --ignored --nocapture # run checked-in UI fixture corpus (canonical tool-group + compatibility/config cases)
nix flake check                               # CI checks, including dedicated nix UI fixture coverage
```

### Project structure

```
src/
├── lib.rs                     # register_lints entry point
├── safety/                    # 15 lints: assertions, panics, errors, defaults, bounds
├── naming/                    # 5 lints: booleans, acronyms, positive predicates, units
├── structure/                 # 8 lints: length, conditions, params, casts, public APIs
├── purity/                    # 3 lints: wall-clock boundaries, verified modules, time
└── concurrency/               # 1 lint: lock ordering
ui_tests/                      # checked-in dylint UI fixtures exercised by nix checks
                               # includes canonical tool-group, compatibility, and *_custom/ config fixtures
openspec/                      # spec-driven change management
```

### Adding a new lint

1. Create `src/<domain>/<lint_name>.rs`
2. Use `declare_tool_lint!(pub tigerstyle::...)`
3. Implement `EarlyLintPass` (syntax-only) or `LateLintPass` (needs types)
4. Register in `src/lib.rs` via the explicit inventory / pass wiring
5. Add a UI test fixture in `ui_tests/`
6. If the lint is configurable, add a `ui_tests/*_custom/` fixture with a local `Cargo.toml` + `dylint.toml` override

## License

AGPL-3.0-or-later
