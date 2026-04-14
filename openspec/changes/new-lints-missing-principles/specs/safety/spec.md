## ADDED Requirements

### Requirement: Explicit Defaults Detection

The lint MUST warn on `Default::default()` calls where the resulting type
is defined in an external crate.

The lint MUST suggest constructing the value with explicit field values.

The lint MUST NOT fire on types defined in the current crate.

The lint MUST NOT fire in `#[cfg(test)]` contexts.

#### Scenario: External type default

- GIVEN `let config = reqwest::ClientBuilder::default();`
- WHEN the lint runs
- THEN it warns: "implicit default on external type — construct with explicit values"

#### Scenario: Own type default

- GIVEN `let state = MyState::default();` where `MyState` is defined in the current crate
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Unbounded Channel Detection

The lint MUST warn on calls to `tokio::sync::mpsc::unbounded_channel()`,
`std::sync::mpsc::channel()`, and `crossbeam::channel::unbounded()`.

The lint MUST suggest using the bounded alternative with explicit capacity.

#### Scenario: Tokio unbounded channel

- GIVEN `let (tx, rx) = tokio::sync::mpsc::unbounded_channel();`
- WHEN the lint runs
- THEN it warns: "unbounded channel — use `mpsc::channel(capacity)` with explicit bound"

#### Scenario: Bounded channel

- GIVEN `let (tx, rx) = tokio::sync::mpsc::channel(100);`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Unbounded Collection Growth In Loops

The lint MUST warn on `.push()`, `.insert()`, or `.extend()` calls on
`Vec`, `HashMap`, `BTreeMap`, or `VecDeque` inside `loop`, `while`, or
`for` bodies when the collection was not created with `with_capacity()`.

The lint MUST NOT fire on collection operations outside of loops.

The lint MUST NOT fire when the collection was created with
`::with_capacity()` or `::from_iter()`.

#### Scenario: Push in loop without capacity

- GIVEN:
  ```rust
  let mut items = Vec::new();
  loop { items.push(read_item()); }
  ```
- WHEN the lint runs
- THEN it warns: "unbounded collection growth in loop — use `Vec::with_capacity()` or add a size limit"

#### Scenario: Push outside loop

- GIVEN `let mut v = Vec::new(); v.push(1);`
- WHEN the lint runs
- THEN it does NOT fire
