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

### Requirement: Public Tiger Style Assertion Review Guide

The project MUST ship a public assertion review guide at
`docs/assertion-review.md` that translates Tiger Style assertion philosophy
into a repeatable review checklist.

That guide MUST cover:

- why these checklist sections matter instead of presenting them as slogans
- assertions on arguments, return values, and invariants
- pair assertions across boundaries
- positive and negative space
- implication assertions and split checks
- compile-time assertions
- invalid-path testing
- explicit fail-fast framing for programmer errors
- explicit guidance that assertion philosophy is not only about raw density

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question.

That guide MUST keep these exact second-level headings:

- `## Assertion targets: arguments, returns, invariants`
- `## Pair assertions across boundaries`
- `## Positive and negative space`
- `## Implication assertions and split checks`
- `## Compile-time assertions`
- `## Invalid-path testing`
- `## Reviewer prompts and failure handling`

Section ownership stays explicit:

- `## Assertion targets: arguments, returns, invariants` owns function
  contract checks.
- `## Pair assertions across boundaries` owns caller/callee, write/read, and
  encode/decode pairing.
- `## Positive and negative space` owns valid/invalid boundary reasoning.
- `## Implication assertions and split checks` owns implication-style checks
  and split compound assertions.
- `## Compile-time assertions` owns constant relations and design-time checks.
- `## Invalid-path testing` owns invalid-path and edge-case test planning.
- `## Reviewer prompts and failure handling` owns why the checklist matters,
  fail-fast framing for programmer errors, and the rule that assertion
  philosophy is not only about raw density.

#### Scenario: Guide keeps required headings and order

- GIVEN the guide has been drafted
- WHEN deterministic verification reads `docs/assertion-review.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Reviewing a boundary with caller and callee checks

- GIVEN a maintainer is reviewing a function, serializer, or storage boundary
- WHEN they follow `docs/assertion-review.md`
- THEN the guide prompts assertions on inputs, outputs, and invariants
- AND it prompts paired checks across the relevant boundary
- AND it keeps fail-fast framing visible for programmer errors

### Requirement: Copyable Assertion Checklist Template

The project MUST ship a copyable assertion checklist template at
`templates/assertion-checklist.md`.

That template MUST include fill-in fields for:

- the function, boundary, or invariant under review
- assertions on inputs, outputs, and invariants
- paired assertion sites
- positive and negative space cases
- implications and split assertions
- compile-time assertions
- invalid-path tests
- open risks and follow-ups

Every template section MUST stay a fill-in field or reviewer prompt rather
than free-form prose.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template.

That template MUST keep these exact second-level headings:

- `## Function, boundary, or invariant under review`
- `## Assertions on inputs, outputs, and invariants`
- `## Paired assertion sites`
- `## Positive and negative space cases`
- `## Implications and split assertions`
- `## Compile-time assertions`
- `## Invalid-path tests`
- `## Open risks and follow-ups`

#### Scenario: Template keeps required headings and order

- GIVEN the template has been drafted
- WHEN deterministic verification reads `templates/assertion-checklist.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Planning invalid-path tests

- GIVEN a maintainer needs to review or plan invalid-path coverage
- WHEN they copy `templates/assertion-checklist.md`
- THEN they can record paired checks, negative-space cases, and invalid-path
  tests without inventing a format first

### Requirement: Saved Deterministic Proof Artifacts For Assertion Kit

The implementation MUST save deterministic verification artifacts at these
paths:

- `openspec/changes/add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- `openspec/changes/add-assertion-review-kit/evidence/assertion-template-headings.txt`
- `openspec/changes/add-assertion-review-kit/evidence/readme-assertion-links.txt`
- `openspec/changes/add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- `openspec/changes/add-assertion-review-kit/evidence/human-assertion-review-labels.txt`

Those artifacts MUST capture these exact saved outputs:

- `assertion-guide-headings.txt` prints the full list of detected guide
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `assertion-template-headings.txt` prints the full list of detected template
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `readme-assertion-links.txt` prints `exists:docs/assertion-review.md`,
  `exists:templates/assertion-checklist.md`, then the two exact README link
  lines.
- `contributor-assertion-links.txt` prints `exists:docs/assertion-review.md`,
  `exists:templates/assertion-checklist.md`,
  `exists:openspec/specs/safety/spec.md`, then the three exact contributor-doc
  link lines.
- `human-assertion-review-labels.txt` prints these exact lines in order after
  exact-once validation succeeds, and it is generated only after
  `openspec/changes/add-assertion-review-kit/evidence/human-assertion-review.txt`
  already exists:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md:1`
  - `question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading:1`
  - `question 2: pass - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings:1`
  - `question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose:1`
  - `question 4: pass - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density:1`
  - `verdict: pass:1`

#### Scenario: Saved guide heading proof is reviewable

- GIVEN guide heading verification has passed
- WHEN a maintainer opens `openspec/changes/add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- THEN the file exists
- AND it prints the full detected guide heading list in the required order

#### Scenario: Saved template heading proof is reviewable

- GIVEN template heading verification has passed
- WHEN a maintainer opens `openspec/changes/add-assertion-review-kit/evidence/assertion-template-headings.txt`
- THEN the file exists
- AND it prints the full detected template heading list in the required order

#### Scenario: Saved README link proof is reviewable

- GIVEN README link verification has passed
- WHEN a maintainer opens `openspec/changes/add-assertion-review-kit/evidence/readme-assertion-links.txt`
- THEN the file exists
- AND it prints the required `exists:` lines plus the two exact README link lines

#### Scenario: Saved contributor link proof is reviewable

- GIVEN contributor-guide link verification has passed
- WHEN a maintainer opens `openspec/changes/add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- THEN the file exists
- AND it prints the required `exists:` lines plus the three exact contributor-guide link lines

#### Scenario: Saved review-label proof is reviewable

- GIVEN human review label verification has passed
- WHEN a maintainer opens `openspec/changes/add-assertion-review-kit/evidence/human-assertion-review-labels.txt`
- THEN the file exists
- AND it prints the exact-once label counts for the required `pass` review record lines

### Requirement: Human Review Record For Assertion Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/add-assertion-review-kit/evidence/human-assertion-review.txt`
for the published guide and template. Each required label below MUST appear
exactly once in that file.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md`
- `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
- `question 2: <pass|fail> - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings`
- `question 3: <pass|fail> - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose`
- `question 4: <pass|fail> - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 4` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the docs are corrected.

#### Scenario: Reviewer signs off assertion coverage

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  paths, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/add-assertion-review-kit/evidence/human-assertion-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 4` are all `pass`
- AND `verdict` is `pass`

