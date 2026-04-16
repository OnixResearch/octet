## Context

Dylint lints emit diagnostics through the rustc `Diagnostic` API, which
supports structured suggestions with `Applicability` levels. The
`span_suggestion` and `multipart_suggestion` methods let tigerstyle attach
reviewable edits directly to a lint diagnostic. `cargo fix` only auto-applies
`MachineApplicable` suggestions, so this change needs an explicit contract for
what counts as safe.

For this change, the supported automatic-application proof path is `cargo fix`
run inside the repo's nix devshell. `rustfix` is not a second required
acceptance path.

## Goals / Non-Goals

**Goals:** Add structured suggestions to lints where tigerstyle can express a
reviewable next edit, while keeping the applicability tier honest for each
pattern.

**Non-Goals:** Auto-fixing structural issues like `function_length` or
`assertion_density`. Those require human judgment about where to split
or what to assert.

## Decisions

### 1. Three tiers of applicability

**Choice:**
- `MachineApplicable`: `compound_assertion` for validated top-level
  conjunction rewrites (`assert!(a && b)`)
- `MaybeIncorrect`: declaration-only rename suggestions for `bool_naming` and
  `acronym_style`, plus `unchecked_division`, `raw_arithmetic_overflow`,
  `unchecked_narrowing`, and `platform_dependent_cast`
- `HasPlaceholders`: `unjustified_allow` and `numeric_units`

**Rationale:**
- `MachineApplicable` must preserve meaning for the validated pattern and keep
  the proof fixture compiling after application.
- Declaration-only renames do not rewrite use sites, so they cannot honestly
  claim auto-apply safety unless tigerstyle later grows full rename coverage for
  the validated scope.
- Guard and conversion suggestions may change control flow or error handling, so
  they stay review-required.
- Placeholder suggestions still need a maintainer-selected unit or real
  justification text.

### 2. Compound assertion autofix is limited to safe conjunction cases

**Choice:** Use `multipart_suggestion` only for validated top-level
`assert!(a && b)` cases. Top-level `assert!(a || b)` stays diagnostic-only or a
non-machine-applicable suggestion until the project proves a
semantics-preserving rewrite for that exact pattern.

**Rationale:** Replacing `assert!(a || b)` with two separate assertions would
strengthen the condition and therefore overclaim safety.

**Implementation:** Replace the full macro span with two assertions,
preserving the original indentation of the source line.

### 3. Rename suggestions stay declaration-scoped unless full propagation exists

**Choice:** `bool_naming` and `acronym_style` suggestions target the
identifier span at the declaration site unless implementation work also rewrites
all affected use sites needed to keep the validated fixture compiling.

**Rationale:** A declaration-only rename is still useful guidance, but it is a
review-required suggestion rather than an honest `MachineApplicable` autofix.
If later work adds complete same-scope propagation, the applicability can be
upgraded with matching proof.

## Risks / Trade-offs

**[Incorrect auto-fixes]** → A `MachineApplicable` suggestion that produces
broken code would undermine trust. Mitigation: require `.fixed` proof for every
machine-applicable pattern plus one saved nix-devshell `cargo fix` transcript.

**[Applicability drift]** → Proposal, tasks, and implementation can easily
start overclaiming safety for rename or disjunction cases. Mitigation: keep the
spec tier contract explicit and add guardrail fixtures for those patterns.

**[Indentation breakage]** → Multi-line suggestions may not match local
indentation style. Mitigation: copy indentation from the original span's
leading whitespace.
