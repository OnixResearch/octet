## Phase 1: Machine-applicable fixes

- [ ] `compound_assertion`: multipart suggestion to split `assert!(a && b)` into two
- [ ] `bool_naming`: suggest `is_` prefix rename at declaration span
- [ ] `acronym_style`: suggest single-word capitalization at type name span
- [ ] UI fixtures with `.fixed` files verifying auto-fix produces compilable code

## Phase 2: Maybe-incorrect suggestions

- [ ] `unchecked_division`: suggest `assert!(b != 0);` insertion
- [ ] `raw_arithmetic_overflow`: suggest `.checked_mul()` / `.saturating_add()` replacement
- [ ] `unchecked_narrowing`: suggest `u32::try_from(value)?` replacement
- [ ] `platform_dependent_cast`: suggest `usize::try_from(value)?` replacement
- [ ] `unjustified_allow`: suggest `// TODO: add justification` comment insertion
- [ ] UI fixtures verifying suggestion text in `.stderr`

## Phase 3: Numeric units placeholder

- [ ] `numeric_units`: suggest unit suffix with placeholder (`timeout` → `timeout_{ms,secs,us}`)
- [ ] UI fixture verifying placeholder suggestion

## Phase 4: Validation

- [ ] Verify `cargo fix` applies machine-applicable suggestions cleanly
- [ ] Update all `.stderr` expectations to include suggestion text
- [ ] Test that `.fixed` files compile without warnings
