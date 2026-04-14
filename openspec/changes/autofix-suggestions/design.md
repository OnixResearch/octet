## Context

Dylint lints emit diagnostics through the rustc `Diagnostic` API, which
supports structured suggestions with `Applicability` levels. The
`span_suggestion` method takes the replacement text, the span to replace,
and an applicability level. `cargo fix` applies `MachineApplicable`
suggestions automatically.

## Goals / Non-Goals

**Goals:** Add structured suggestions to lints where the fix is mechanical
enough to express as a span replacement.

**Non-Goals:** Auto-fixing structural issues like `function_length` or
`assertion_density`. Those require human judgment about where to split
or what to assert.

## Decisions

### 1. Three tiers of applicability

**Choice:**
- `MachineApplicable`: `compound_assertion` (split), `bool_naming`
  (add prefix), `acronym_style` (recapitalize)
- `MaybeIncorrect`: `unchecked_division` (add assert), `raw_arithmetic_overflow`
  (suggest checked op), `unchecked_narrowing` (suggest try_from),
  `platform_dependent_cast` (suggest try_from)
- `HasPlaceholders`: `unjustified_allow` (insert TODO comment),
  `numeric_units` (suggest suffix but can't know which unit)

**Rationale:** Machine-applicable fixes must be semantically equivalent.
Maybe-incorrect fixes change behavior (adding an assert, changing
arithmetic semantics). Placeholder fixes need human input to complete.

### 2. Multi-span suggestions for compound_assertion

**Choice:** Use `multipart_suggestion` to replace a single `assert!(a && b)`
with two statements.

**Rationale:** Single `span_suggestion` can only replace one contiguous
span. Splitting an assertion into two statements requires replacing the
macro call and inserting a newline plus the second assertion.

**Implementation:** Replace the full macro span with
`assert!(a);\nassert!(b);`, preserving the indentation of the original
line.

### 3. Rename suggestions use the declaration span

**Choice:** `bool_naming` and `acronym_style` suggestions target the
identifier span at the declaration site, not usage sites.

**Rationale:** Renaming at usage sites would require a full-project rename
which is beyond a single lint's scope. The suggestion at the declaration
site is the right signal; `cargo fix` with `--edition` or IDE rename
handles propagation.

## Risks / Trade-offs

**[Incorrect auto-fixes]** → A `MachineApplicable` suggestion that
produces broken code would undermine trust. Mitigation: UI test `.fixed`
files verify that applying the suggestion produces compilable, correct
code.

**[Indentation breakage]** → Multi-line suggestions may not match local
indentation style. Mitigation: copy indentation from the original span's
leading whitespace.
