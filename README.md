# tigerstyle

Custom [dylint](https://github.com/trailofbits/dylint) lints enforcing
[Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
coding principles for Rust.

25 lints across 5 domains: safety, naming, structure, purity, concurrency.

## Lint Catalog

### Safety (11 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `compound_assertion` | **warn** | `assert!(a && b)` should be split into separate assertions |
| `ignored_result` | **warn** | `let _ = expr` silently discards a `Result` |
| `no_unwrap` | **warn** | `.unwrap()`/`.expect()` in production code |
| `no_panic` | **warn** | `panic!()`/`todo!()`/`unimplemented!()` in production code |
| `no_recursion` | **warn** | Function calls itself recursively |
| `unchecked_narrowing` | **warn** | `as u32` from wider type without bounds check |
| `unchecked_division` | **warn** | `a / b` where `b` is a variable without zero guard |
| `unbounded_loop` | **warn** | `loop {}` without visible iteration bound |
| `catch_all_on_enum` | **warn** | `_ => false` catch-all on enum match hides new variants |
| `assertion_density` | allow | 20+ line function with fewer than 2 assertions |
| `raw_arithmetic_overflow` | allow | Unprotected `+`/`-`/`*` on unsigned integers |

### Naming (4 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `acronym_style` | **warn** | `RPCHandler` should be `RpcHandler` |
| `bool_naming` | **warn** | Boolean binding without `is_`/`has_`/`should_`/`can_` prefix |
| `numeric_units` | allow | Bare `timeout` without unit suffix (`_ms`, `_secs`) |
| `float_for_currency` | allow | `f64` in cost/price/budget struct fields |

### Structure (6 lints)

| Lint | Default | Description |
|------|---------|-------------|
| `function_length` | **warn** | Function body exceeds 70 non-blank, non-comment lines |
| `nested_conditionals` | **warn** | `if`/`match` nesting exceeds 3 levels |
| `platform_dependent_cast` | **warn** | `as usize` from fixed-width integer is platform-dependent |
| `compound_condition` | allow | `if` condition with 3+ boolean operators |
| `unjustified_allow` | allow | `#[allow(...)]` without justification comment |
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
- [cargo-dylint](https://github.com/trailofbits/dylint) and `dylint-link`

```bash
cargo install cargo-dylint dylint-link
```

### Running lints

Add tigerstyle as a dependency in your workspace:

```toml
# Cargo.toml or workspace Cargo.toml
[workspace.metadata.dylint]
libraries = [
    { git = "https://github.com/your-org/tigerstyle" },
]
```

Then run:

```bash
cargo dylint tigerstyle
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

Place a `dylint.toml` in your workspace root to tune thresholds:

```toml
[tigerstyle]
function_length_max = 100          # default: 70
nesting_max_depth = 4              # default: 3
assertion_density_min_lines = 30   # default: 20
assertion_density_min_assertions = 3  # default: 2
acronym_allowlist = ["IO", "ID"]   # default: []
```

All fields are optional — missing fields use the defaults shown above.

## Development

```bash
nix develop                    # enter devshell
cargo check                    # verify compilation
cargo test                     # run unit tests + UI tests
nix flake check                # full CI (build + test + clippy + fmt)
```

### Project structure

```
src/
├── lib.rs                     # register_lints entry point
├── safety/                    # 11 lints: assertions, panics, errors, bounds
├── naming/                    # 4 lints: booleans, acronyms, units
├── structure/                 # 6 lints: length, conditions, params, casts
├── purity/                    # 3 lints: wall-clock boundaries, verified modules, time
└── concurrency/               # 1 lint: lock ordering
ui_tests/                      # dylint UI test fixtures
openspec/                      # spec-driven change management
```

### Adding a new lint

1. Create `src/<domain>/<lint_name>.rs`
2. Use `declare_lint!` + `declare_lint_pass!`
3. Implement `EarlyLintPass` (syntax-only) or `LateLintPass` (needs types)
4. Register in `src/lib.rs` via `register_lints`
5. Add a UI test fixture in `ui_tests/`

## License

AGPL-3.0-or-later
