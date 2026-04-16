# Safety Specification

## Purpose

Tiger Style safety requirements for lint enforcement: panic avoidance,
explicit defaults, bounded queues and resource growth, checked arithmetic,
and reviewable failure handling.
## Requirements
### Requirement: Panic Macro In Production Code

The system MUST detect `panic!()`, `todo!()`, `unimplemented!()`, and
`unreachable!()` in production code.

The system MUST continue to support a narrow temporary-stub escape hatch for
`todo!()` after `no_panic` becomes a true tool lint.

Because `#[allow(tigerstyle::no_panic)]` suppresses the `no_panic` lint at the
rustc level, the system MUST audit that escape hatch with a companion lint,
`unjustified_no_panic_allow`, so misuse still warns.

The system MUST NOT fire either lint for `todo!()` when the call is inside a
function or block explicitly allowed with `#[allow(tigerstyle::no_panic)]` and
an adjacent justification comment.

For this rule, an adjacent justification comment is a `// ...` comment on the
same line as the allow attribute or on the immediately preceding non-empty
line. Blank lines, doc comments, and block comments do NOT count.

The system MUST warn via `unjustified_no_panic_allow` when any of these hold:
- `#[allow(tigerstyle::no_panic)]` appears without that adjacent `//`
  justification comment
- `#[allow(tigerstyle::no_panic)]` is used around `panic!()`
- `#[allow(tigerstyle::no_panic)]` is used around `unimplemented!()`
- `#[allow(tigerstyle::no_panic)]` is used around `unreachable!()`

#### Scenario: Justified todo allow

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint suite runs
- THEN neither `no_panic` nor `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Justified block todo allow

- GIVEN a block annotated with `#[allow(tigerstyle::no_panic)]` and an
  adjacent `//` justification comment that contains `todo!(...)`
- WHEN the lint suite runs
- THEN neither `no_panic` nor `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Unjustified todo still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { todo!(...) }` without that
  adjacent `//` justification comment
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Panic escape still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { panic!(...) }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Unimplemented escape still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { unimplemented!(...) }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Unreachable escape still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { unreachable!(...) }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

### Requirement: No Panic Allow Hygiene

The system MUST warn when `#[allow(tigerstyle::no_panic)]` remains on an item or
block that contains no `todo!()`, `panic!()`, `unimplemented!()`, or
`unreachable!()` call covered by that escape hatch.

This warning MUST use `unjustified_no_panic_allow` so the existing companion
lint remains the single audit path for `no_panic` escapes.

#### Scenario: Live justified todo allow stays quiet

- GIVEN `// justified stub while API lands`
  `#[allow(tigerstyle::no_panic)] fn helper() { todo!("later"); }`
- WHEN the lint suite runs
- THEN neither `no_panic` nor `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Orphaned item allow

- GIVEN `#[allow(tigerstyle::no_panic)] fn helper() { do_work(); }`
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: Orphaned block allow

- GIVEN a block annotated with `#[allow(tigerstyle::no_panic)]` whose body has
  no panic-family macro
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

#### Scenario: `cfg_attr`-expanded orphan allow

- GIVEN `cfg_attr(...)` expands into a direct `#[allow(tigerstyle::no_panic)]`
  on a supported item or block, and that annotated body has no panic-family
  macro
- WHEN the lint suite runs
- THEN `unjustified_no_panic_allow` emits a diagnostic

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

