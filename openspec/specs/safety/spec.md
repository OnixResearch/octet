# Safety Specification

## Purpose

Tiger Style safety requirements for lint enforcement: panic avoidance,
explicit defaults, bounded queues and resource growth, checked arithmetic,
and reviewable failure handling.
## Requirements
### Requirement: Panic Macro In Production Code

The system MUST detect `panic!()`, `unimplemented!()`, and `unreachable!()` in
production code through `no_panic`.

The system MUST detect `todo!()` in production code through a dedicated safety
lint, `no_todo`.

The system MUST register `no_todo` as a warn-by-default safety tool lint in
canonical `tigerstyle::no_todo` form, with the repo's current one-member bare
alias path continuing to resolve supported bare source/config spellings.

The system MUST keep the repo's supported test-context suppressions for both
lints, including `#[cfg(test)]`, `#[test]`, and the shared file/context
heuristics already used by tigerstyle.

The system MUST NOT treat `#[allow(tigerstyle::no_panic)]` or
`#[allow(no_panic)]` as the documented escape hatch for `todo!()` once the
split lands.

#### Scenario: `todo!()` emits the dedicated lint

- GIVEN production code that invokes `todo!(...)`
- WHEN the lint suite runs after the split
- THEN `no_todo` emits the diagnostic
- AND `no_panic` does not claim that same `todo!()` call

#### Scenario: panic-family macros stay on `no_panic`

- GIVEN production code that invokes `panic!(...)`, `unimplemented!(...)`, or
  `unreachable!(...)`
- WHEN the lint suite runs after the split
- THEN `no_panic` emits the diagnostic
- AND `no_todo` does not claim those macros

#### Scenario: legacy tool-scoped `no_panic` allow does not hide `todo!()`

- GIVEN a `todo!(...)` stub under `#[allow(tigerstyle::no_panic)]`
- WHEN the lint suite runs after the split
- THEN `no_todo` still emits the diagnostic
- AND the maintainer must migrate that escape hatch to `no_todo`

#### Scenario: legacy bare `no_panic` allow does not hide `todo!()`

- GIVEN a `todo!(...)` stub under `#[allow(no_panic)]`
- WHEN the lint suite runs after the split
- THEN `no_todo` still emits the diagnostic
- AND the maintainer must migrate that escape hatch to `no_todo`

### Requirement: Explicit Defaults Detection

The lint MUST warn when a call resolves to the `Default` trait's `default`
function and the resulting concrete type is defined in a different crate than
the current crate.

For this rule, sibling workspace crates count as external crates because they
compile as distinct crate identities.

This rule MUST cover these user-visible forms when they resolve to the
`Default` trait implementation:
- `Default::default()`
- `<T as Default>::default()`
- `T::default()`
- struct-update shorthand such as `Config { timeout_ms: 5_000, ..Default::default() }`
- attribute- or derive-driven shorthand such as `#[serde(default)]` when the
  expanded call resolves to a foreign `Default::default()`

The lint MUST use resolved trait/function identity rather than path-text string
matching.

The lint MUST NOT fire on types defined in the current crate.

The lint MUST NOT fire in the repo's supported test contexts, including
`#[cfg(test)]`, `#[test]`, and source-file test heuristics already used by the
shared test-context helper.

#### Scenario: External type default

- GIVEN `let config = reqwest::ClientBuilder::default();`
- WHEN the lint runs
- THEN it warns that the code relies on an implicit default from an external
  crate

#### Scenario: Sibling workspace type default

- GIVEN `let cfg: shared_types::Config = Default::default();` where
  `shared_types` is a different crate in the same workspace
- WHEN the lint runs
- THEN it warns

#### Scenario: Struct update shorthand

- GIVEN `let cfg = ClientCfg { retries: 3, ..Default::default() };` where the
  base `Default::default()` resolves to an external type
- WHEN the lint runs
- THEN it warns

#### Scenario: Own type default

- GIVEN `let state = MyState::default();` where `MyState` is defined in the
  current crate
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Unbounded Channel Detection

The lint MUST warn on calls whose resolved callee identity is one of these
unbounded channel constructors:
- `std::sync::mpsc::channel`
- `crossbeam::channel::unbounded`, whose resolved definition path is
  `crossbeam_channel::channel::unbounded`
- Tokio's unbounded MPSC constructor, whose resolved definition path is
  `tokio::sync::mpsc::unbounded::unbounded_channel`

The lint MUST match by resolved function identity or definition path, not only
by the public re-export spelling that appears in source.

The lint MUST suggest using the bounded alternative with explicit capacity.

#### Scenario: Tokio unbounded channel

- GIVEN `let (tx, rx) = tokio::sync::mpsc::unbounded_channel();`
- WHEN the lint runs
- THEN it warns to use the bounded channel constructor with explicit capacity

#### Scenario: Bounded channel

- GIVEN `let (tx, rx) = tokio::sync::mpsc::channel(100);`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Unbounded Collection Growth In Loops

The lint MUST warn only on narrow same-block cases where:
- a local binding of type `Vec`, `HashMap`, `BTreeMap`, or `VecDeque` is
  initialized empty in the same block
- that same binding is grown with `.push()`, `.insert()`, or `.extend()`
  inside a `loop`, `while`, or `for` body in that same block
- and the same binding has no prior reservation or explicit growth bound that
  makes the loop growth reviewably finite

For this rule, an accepted prior reservation is one of these on the same
binding before the loop-growth site:
- `::with_capacity(...)`
- `reserve(...)`
- `reserve_exact(...)`
- `try_reserve(...)`

The lint MUST NOT fire when dataflow or aliasing is unclear, including bindings
created outside the current block or mutated through a different binding.

The lint MUST NOT fire on collection operations outside loops.

The lint MUST NOT fire when the same binding is guarded by an explicit local
capacity/length bound on that same binding (for example `items.len() < limit`
or `items.len() < items.capacity()`) that makes the growth finite and
reviewable.

A finite loop bound that is not expressed through the same binding's local
reservation or `len`/`capacity` check does NOT suppress the lint.

#### Scenario: Push in loop without capacity

- GIVEN:
  ```rust
  let mut items = Vec::new();
  loop { items.push(read_item()); }
  ```
- WHEN the lint runs
- THEN it warns that the loop grows the collection without a prior bound or
  reservation

#### Scenario: Reserved before loop

- GIVEN:
  ```rust
  let mut items = Vec::new();
  items.reserve(128);
  while has_more() { items.push(read_item()); }
  ```
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Finite range loop still warns without a collection-local bound

- GIVEN:
  ```rust
  let mut items = Vec::new();
  for value in 0..10 { items.push(value); }
  ```
- WHEN the lint runs
- THEN it still warns because the loop has no prior reservation and no
  collection-local `len`/`capacity` guard

#### Scenario: Ambiguous outer binding

- GIVEN a collection binding created outside the current block and later grown
  inside a loop
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Sentinel Fallback Detection

The lint MUST default to `warn`.

The lint MUST warn when code uses `Option::unwrap_or`,
`Option::unwrap_or_else`, `Result::unwrap_or`, or `Result::unwrap_or_else` to
replace absence or failure with an integer extreme sentinel for the same target
integer type.

For the first rollout, the accepted sentinel forms are `<int>::MAX` and
`<int>::MIN`, where `<int>` is one of `u8..u128`, `i8..i128`, `usize`, or
`isize`.

For this rule, `same target integer type` means the sentinel constant's type
matches the type returned by the `unwrap_or` / `unwrap_or_else` call.

The lint MUST use receiver type and resolved constant identity rather than raw
source-text matching.

The lint MUST explain that the sentinel masks absence or failure and MUST
recommend an explicit branch, propagated error/option state, or a domain-tagged
fallback representation.

The lint MUST fire at the `unwrap_or` / `unwrap_or_else` call site regardless of
how the resulting value is later used, including when that value is immediately
formatted into a string, key, or identifier.

The lint MUST NOT fire when code branches explicitly on the `Option` or
`Result` instead of forcing an extreme sentinel.

The lint MUST NOT fire on non-extreme defaults such as `0`, `1`, or other
non-`MIN` / `MAX` fallback expressions.

The lint MUST continue to respect normal rustc allow / deny handling,
including `#[allow(sentinel_fallback)]`.

The lint MUST NOT fire in the repo's supported test contexts.

#### Scenario: Fallible conversion masked by max sentinel

- GIVEN `format!("recent_{}", u64::try_from(command_index).unwrap_or(u64::MAX))`
- WHEN the lint runs
- THEN it warns that the code masks conversion failure with an extreme sentinel

#### Scenario: Optional index masked by max sentinel

- GIVEN `let idx = maybe_index.unwrap_or(u32::MAX);`
- WHEN the lint runs
- THEN it warns that the code masks absence with an extreme sentinel

#### Scenario: Side-effect closure masked by min sentinel

- GIVEN `let idx = index_result.unwrap_or_else(|err| { log::warn!("{err}"); i64::MIN });`
- WHEN the lint runs
- THEN it warns that the code still masks absence or failure with an extreme sentinel

#### Scenario: Explicit branch stays quiet

- GIVEN code that uses `match` or `if let` to handle the missing or failing
  case explicitly before formatting or returning a value
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-extreme default stays quiet

- GIVEN `let idx = maybe_index.unwrap_or(0);`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-extreme closure default stays quiet

- GIVEN `let idx = maybe_index.unwrap_or_else(|_| 0);`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Test-context sentinel fallback stays quiet

- GIVEN `unwrap_or(u64::MAX)` appears inside one of the repo's supported test
  contexts
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: No Todo Allow Hygiene

The system MUST support a narrow temporary-stub escape hatch for `todo!()` via
`#[allow(tigerstyle::no_todo)]` and the supported bare compatibility spelling
`#[allow(no_todo)]`.

The system MUST register `unjustified_no_todo_allow` as a warn-by-default
safety lint in canonical tool-scoped form, while the repo's current bare alias
path continues to resolve the supported bare source/config spellings.

The system MUST NOT fire either `no_todo` or `unjustified_no_todo_allow` when a
`todo!()` call sits inside a function or block explicitly allowed with either
`#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` and an adjacent
justification comment.

For this rule, an adjacent justification comment is a `// ...` comment on the
same line as the allow attribute or on the immediately preceding non-empty
line. Blank lines, doc comments, and block comments do NOT count.

The system MUST warn via `unjustified_no_todo_allow` when any of these hold:
- `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` appears without that
  adjacent `//` justification comment
- `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` appears around
  `panic!()`, `unimplemented!()`, or `unreachable!()` instead of a `todo!()`
  stub
- `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` remains on an item or
  block after the covered `todo!()` stub is gone
- `cfg_attr(...)` expands into one of those supported direct allow attrs on a
  covered item or block, and the resulting expanded attr is unjustified or
  orphaned

#### Scenario: justified tool-scoped `no_todo` allow stays quiet

- GIVEN `#[allow(tigerstyle::no_todo)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint suite runs
- THEN neither `no_todo` nor `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: justified bare `no_todo` allow stays quiet

- GIVEN `#[allow(no_todo)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint suite runs
- THEN neither `no_todo` nor `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: unjustified `no_todo` allow warns

- GIVEN `#[allow(tigerstyle::no_todo)] fn stub() { todo!(...) }` without that
  adjacent `//` justification comment
- WHEN the lint suite runs
- THEN `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: orphaned or misapplied `no_todo` allow warns

- GIVEN `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` either wraps a
  non-`todo!()` panic-family macro or covers no remaining `todo!()` call at all
- WHEN the lint suite runs
- THEN `unjustified_no_todo_allow` emits a diagnostic

#### Scenario: `cfg_attr`-expanded `no_todo` allow is audited

- GIVEN `cfg_attr(...)` expands into a direct `#[allow(tigerstyle::no_todo)]`
  or `#[allow(no_todo)]` on a supported item or block
- WHEN the lint suite runs
- THEN `unjustified_no_todo_allow` applies the same justification and orphan
  checks to that expanded attr

