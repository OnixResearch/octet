## Why

Current lint diagnostics emit a one-line warning and sometimes a suggestion.
They do not explain the Tiger Style principle being enforced, link to
documentation, or show the approved pattern. A developer seeing
`ambient_clock` for the first time has to find and read the Tiger Style
reference to understand what to do. The downstream rollout playbook includes
extensive pattern guidance (`ambient_clock` boundary playbook, narrow allow
rules) that should be surfaced in the diagnostic itself.

## What Changes

- Add a `help` note to every lint diagnostic that names the Tiger Style
  principle and shows the approved pattern
- Add a `note` with a short URL or file reference to the relevant section
  of the Tiger Style guide
- For lints with common fix patterns, include a concrete before/after
  example in the `help` text

Example improved diagnostic for `ambient_clock`:

```
warning: direct wall-clock read in production code
  --> src/coordination/timeout.rs:42:18
   |
42 |     let now = Instant::now();
   |               ^^^^^^^^^^^^^^
   |
   = help: Tiger Style: inject time as a parameter for core logic,
           or isolate the clock read in a shell-boundary helper with
           `#[allow(tigerstyle::ambient_clock, reason = "...")]`
   = note: see Tiger Style guide: Functional Core / Imperative Shell
```

Example improved diagnostic for `compound_assertion`:

```
warning: compound assertion — split for better diagnostics
  --> src/verified/lock.rs:15:5
   |
15 |     assert!(a > 0 && b > 0);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: split into: `assert!(a > 0); assert!(b > 0);`
           When a compound assertion fails, only the outer expression
           is shown. Split assertions pinpoint which condition failed.
   = note: see Tiger Style guide: Assertion Density
```

- Standardize diagnostic wording across all 26 lints to use consistent
  formatting: `Tiger Style: <principle name>` in help text
- Add lint-group names that match spec domains (`tigerstyle::safety`,
  `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`,
  `tigerstyle::concurrency`) for easier group-level `#[allow]`

## Capabilities

### New Capabilities
- `rich-diagnostics`: Every lint emits help text with principle name,
  approved pattern, and reference pointer
- `lint-groups`: Domain-level lint groups for bulk `#[allow]`/`#[deny]`

### Modified Capabilities
- All 26 lint diagnostics get richer output

## Impact

- **Files**: Every lint source file (diagnostic formatting), `src/lib.rs`
  (lint group registration)
- **APIs**: New lint group IDs
- **Dependencies**: None
- **Testing**: All UI `.stderr` fixtures must be updated to match new
  diagnostic format
