## ADDED Requirements

### Requirement: Usize In Public API

The lint MUST warn on `pub fn` parameters and return types that use
`usize` or `isize`.

The lint MUST suggest using fixed-width integer types (`u32`, `u64`).

The lint MUST NOT fire on trait implementations (signature dictated by
the trait).

The lint MUST NOT fire on `Index`/`IndexMut` implementations.

The lint MUST NOT fire on private functions.

#### Scenario: Public function with usize parameter

- GIVEN `pub fn get_item(index: usize) -> &Item`
- WHEN the lint runs
- THEN it warns: "`usize` in public API — use `u32` or `u64` for cross-platform consistency"

#### Scenario: Trait implementation

- GIVEN `impl Index<usize> for MyVec { fn index(&self, idx: usize) -> &T { ... } }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Too Many Parameters

The lint MUST warn on functions with more than a configurable number of
parameters (default: 5). `self` and `&self` do not count.

The lint MUST suggest grouping parameters into a named struct.

The lint MUST NOT fire on trait implementations.

The lint MUST NOT fire on `#[test]` functions.

The threshold MUST be configurable via `max_parameters` in `dylint.toml`.

#### Scenario: Six parameters

- GIVEN `fn connect(host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str)`
- WHEN the lint runs
- THEN it warns: "function has 6 parameters (limit: 5) — consider an options struct"

#### Scenario: Five parameters

- GIVEN a function with exactly 5 parameters
- WHEN the lint runs
- THEN it does NOT fire (at default threshold)
