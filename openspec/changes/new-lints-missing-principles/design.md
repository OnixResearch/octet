## Context

The Tiger Style reference document and skill cover principles that have
no corresponding lint. The performance spec domain has zero lints (and
likely should stay that way — performance rules are design-time guidance,
not pattern-matchable syntax). But several safety, structure, and naming
principles are pattern-matchable and would catch real bugs.

## Goals / Non-Goals

**Goals:** Add lints for Tiger Style principles that are reliably
detectable through syntax or type analysis.

**Non-Goals:** Performance lints. Performance principles like "napkin math"
and "batch operations" are design-time guidance that can't be reduced to
syntax patterns.

## Decisions

### 1. `explicit_defaults` targets external crate types only

**Choice:** Only warn on `Default::default()` for types defined outside
the current crate.

**Rationale:** A project's own types have known, auditable defaults.
External types can change their defaults across versions. This matches
the Tiger Style principle: "do not rely on library defaults staying
stable."

**Implementation:** Late pass. Resolve the type of the `Default::default()`
call. Check if the type's `DefId` belongs to an external crate.

**Alternative:** Warn on all `Default::default()` — rejected because
it's too noisy for types the project owns and controls.

### 2. `unbounded_channel` covers three ecosystems

**Choice:** Detect `tokio::sync::mpsc::unbounded_channel`,
`std::sync::mpsc::channel` (which is unbounded), and
`crossbeam::channel::unbounded`.

**Rationale:** These are the three most common Rust channel crates. Each
has a bounded alternative.

**Implementation:** Late pass. Match on call paths to the known functions.

### 3. `unbounded_collection_growth` fires only in loops

**Choice:** Only warn on `.push()`, `.insert()`, `.extend()` inside
`loop`, `while`, or `for` bodies where the collection was not created
with `with_capacity`.

**Rationale:** A single `.push()` outside a loop is normal. Unbounded
growth inside a loop without capacity planning is the Tiger Style
violation.

**Implementation:** Early pass. Walk loop bodies for method calls on
collection types. Check if the receiver was created with `::new()` vs
`::with_capacity()`.

**Alternative:** Warn on all `.push()` without capacity — rejected
because it's far too noisy.

### 4. `usize_in_public_api` complements `platform_dependent_cast`

**Choice:** Warn on `pub fn` parameters and return types that use `usize`
or `isize`.

**Rationale:** `platform_dependent_cast` catches `as usize` at cast sites,
but the problem starts earlier when `usize` enters the API surface.
Fixing the API avoids the cast entirely.

**Implementation:** Late pass. Check `pub fn` signatures for `usize`/
`isize` parameters and return types. Skip trait implementations (signature
is dictated by the trait). Skip indexing helpers where `usize` is required
by `Index`/`IndexMut`.

### 5. `too_many_parameters` defaults to 5

**Choice:** Configurable threshold, default 5.

**Rationale:** Tiger Style says "simplify function signatures" and "limit
parameters." Five is a common threshold in other linting tools (clippy's
`too_many_arguments` defaults to 7). Five is more opinionated but matches
Tiger Style's emphasis on options structs.

**Implementation:** Early pass. Count function parameters. Skip `self`.
Configurable via `dylint.toml` `max_parameters`.

### 6. `negated_predicate` checks names, not logic

**Choice:** Warn on function and variable names containing `is_not_`,
`has_no_`, `lacks_`, `not_`, `cant_`, `doesnt_`.

**Rationale:** Tiger Style says "state invariants positively." This is a
naming lint, not a logic lint — it can't detect `!is_ready` usage patterns
but can catch names that encode negation.

**Implementation:** Early pass. Check identifier names against a list of
negative prefixes.

## Risks / Trade-offs

**[New-lint noise in downstream]** → Every new lint starts at warn level
and could be noisy in existing codebases. Mitigation: new lints should
be added to the inventory-first profile, not the starter profile, until
downstream validation proves they're low-noise.

**[Collection-growth false positives]** → `unbounded_collection_growth`
may fire on loops that are already bounded by the iterator (e.g.,
`for x in bounded_iter { vec.push(x); }`). Mitigation: skip `for x in`
loops where the iterator is bounded. Accept some false positives and
let consumers use narrow allows.
