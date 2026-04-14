## Why

Several Tiger Style principles lack lint enforcement entirely. The
performance spec domain has zero lints. Key safety principles around
explicit defaults, unbounded resource usage, and API surface hygiene are
covered in the skill and reference but have no automated detection.

## What Changes

### Safety domain

- **`explicit_defaults`**: Warn on `Default::default()` or `.default()` on
  types from external crates in production code. Tiger Style says "avoid
  implicit defaults — pass explicit options; do not rely on library defaults
  staying stable." Suggest constructing the value with explicit field values.

- **`unbounded_channel`**: Warn on `tokio::sync::mpsc::unbounded_channel()`,
  `std::sync::mpsc::channel()`, and `crossbeam::channel::unbounded()`.
  Tiger Style says "set fixed limits on queues." Suggest bounded alternatives
  with explicit capacity.

- **`unbounded_collection_growth`**: Warn on `.push()`, `.insert()`, or
  `.extend()` on `Vec`, `HashMap`, `BTreeMap`, `VecDeque` inside loop
  bodies without a preceding capacity check or `with_capacity` allocation.
  Tiger Style says "static memory allocation — allocate at startup, avoid
  dynamic allocation after initialization."

### Structure domain

- **`usize_in_public_api`**: Warn on `pub fn` signatures that use `usize`
  or `isize` as parameter or return types. Extends `platform_dependent_cast`
  to catch the problem at API boundaries, not just at cast sites. Suggest
  fixed-width types (`u32`, `u64`).

- **`too_many_parameters`**: Warn on functions with more than 5 parameters
  (configurable). Tiger Style says "simplify function signatures, limit
  parameters." Suggest an options struct.

### Naming domain

- **`negated_predicate`**: Warn on boolean-returning functions or variables
  with names like `is_not_`, `has_no_`, `lacks_`, `!is_`. Tiger Style
  says "state invariants positively — prefer `is_ready` over
  `!is_not_ready`."

## Capabilities

### New Capabilities
- `explicit_defaults`: Detects implicit external-type defaults
- `unbounded_channel`: Detects unbounded channel construction
- `unbounded_collection_growth`: Detects unchecked collection growth in loops
- `usize_in_public_api`: Detects platform-dependent types in public APIs
- `too_many_parameters`: Detects functions with excessive parameter counts
- `negated_predicate`: Detects double-negation and negative predicate names

## Impact

- **Files**: New files in `src/safety/`, `src/structure/`, `src/naming/`,
  plus registration in `src/lib.rs`
- **APIs**: Six new lint IDs
- **Dependencies**: None
- **Testing**: UI fixtures for each new lint, downstream validation against
  Aspen to assess noise level before adding to any rollout profile
