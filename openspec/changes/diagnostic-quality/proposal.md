## Why

Current lint diagnostics emit a one-line warning and sometimes a suggestion.
They do not explain the Tiger Style principle being enforced, point to the
relevant guide section, or show the approved pattern. A developer seeing
`ambient_clock` for the first time has to go hunt for the Tiger Style
reference before they know what "good" looks like. The downstream rollout
kit already carries pattern guidance (`ambient_clock` boundary playbook,
narrow allow rules); diagnostics should surface that guidance directly.

## What Changes

- Add a `help` note to every lint diagnostic that names the Tiger Style
  principle and shows the approved pattern; when a lint has a concrete fix
  pattern, append it inside the same help note as `Example: ...`
- Add a `note` in the form `see Tiger Style guide: <section>` so the
  developer gets a stable section reference instead of a stale URL
- For lints with common fix patterns, include a concrete before/after
  example in the `help` text by appending `Example: ...` to the same help note
- Register lint-group names that match spec domains (`tigerstyle::safety`,
  `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`,
  `tigerstyle::concurrency`) for easier group-level `#[allow]` / `#[deny]`

Example improved diagnostic for `ambient_clock`:

```
warning: direct wall-clock read in production code
  --> src/coordination/timeout.rs:42:18
   |
42 |     let now = Instant::now();
   |               ^^^^^^^^^^^^^^
   |
   = help: Tiger Style (Functional Core): inject time as a parameter for core logic, or isolate the clock read in a shell-boundary helper with a narrow allow reason. Example: `#[allow(tigerstyle::ambient_clock, reason = "shell boundary clock read")] fn read_now() -> Instant { Instant::now() }`
   = note: see Tiger Style guide: Functional Core / Imperative Shell
```

Example improved diagnostic for `compound_assertion`:

```
warning: compound assertion should be split into separate assertions
  --> src/verified/lock.rs:15:5
   |
15 |     assert!(a > 0 && b > 0);
   |     ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: Tiger Style (Assertion Quality): split compound assertions so the failing clause is obvious. Example: `assert!(a > 0); assert!(b > 0);`
   = note: see Tiger Style guide: Assertion Density
```

## Capabilities

### New Capabilities
- `rich-diagnostics`: Every lint emits help text with a principle name,
  approved pattern, and guide-section pointer
- `lint-groups`: Domain-level lint groups for bulk `#[allow]` / `#[deny]`

### Modified Capabilities
- All 25 lint diagnostics get richer output

## Impact

- **Files**: Every lint source file, `src/diagnostics.rs`, `src/lib.rs`,
  UI fixtures, and inventory snapshots
- **APIs**: New lint group IDs
- **Dependencies**: None
- **Testing**: Update all UI `.stderr` fixtures, add group allow/deny UI
  fixtures, and run `cargo test --lib`, `cargo test --test ui -- --ignored --nocapture`,
  and `nix flake check`
