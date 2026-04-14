## Why

None of the 26 lints provide machine-applicable fix suggestions. When a
downstream consumer runs tigerstyle and gets 50 warnings, every fix is
manual. Lints like `compound_assertion`, `bool_naming`, `acronym_style`,
and `numeric_units` have mechanical, predictable fixes that could be
applied automatically via `cargo fix` or `rustfix`.

## What Changes

Add `Applicability::MachineApplicable` or `MaybeIncorrect` suggestions to
lints where the fix is mechanical:

### Machine-applicable (safe to auto-apply)
- `compound_assertion`: split `assert!(a && b)` into `assert!(a); assert!(b);`
- `bool_naming`: suggest `is_` prefix rename
- `acronym_style`: suggest single-word capitalization (`RPCHandler` → `RpcHandler`)
- `numeric_units`: suggest unit suffix (`timeout` → `timeout_ms`)

### Maybe-incorrect (suggested but needs human review)
- `unchecked_division`: suggest `assert!(b != 0);` before the division
- `raw_arithmetic_overflow`: suggest `.checked_mul()` / `.saturating_add()`
- `unchecked_narrowing`: suggest `u32::try_from(value)?`
- `platform_dependent_cast`: suggest `usize::try_from(value)?`
- `unjustified_allow`: insert `// TODO: add justification` comment

### Not auto-fixable (diagnostics only)
- `assertion_density`, `function_length`, `ambiguous_params`,
  `multi_lock_ordering`, `catch_all_on_enum`, `no_recursion` — these
  require structural refactoring

## Capabilities

### New Capabilities
- `autofix`: Lints with mechanical fixes emit structured suggestions that
  `cargo fix` can apply

## Impact

- **Files**: Every lint source file that gets a suggestion added
- **APIs**: No public API changes
- **Dependencies**: None
- **Testing**: UI test `.stderr` files must be updated to include the
  suggestion text; new `.fixed` files for machine-applicable lints
