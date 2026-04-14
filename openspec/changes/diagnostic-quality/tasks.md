## Phase 1: Diagnostic helper

- [ ] Create `src/diagnostics.rs` with `tiger_style_help()` helper
- [ ] Define principle names and reference sections for all 26 lints
- [ ] Add helper to each lint's diagnostic emission

## Phase 2: Safety domain diagnostics

- [ ] `compound_assertion`: principle "Assertion Quality", show split pattern
- [ ] `assertion_density`: principle "Assertion Density", show minimum expectations
- [ ] `unbounded_loop`: principle "Bounded Control Flow", show bounded alternative
- [ ] `ignored_result`: principle "Error Handling", show propagation pattern
- [ ] `unchecked_narrowing`: principle "Safe Narrowing", show `try_from` pattern
- [ ] `unchecked_division`: principle "Explicit Division", show guard pattern
- [ ] `raw_arithmetic_overflow`: principle "Overflow Protection", show checked ops
- [ ] `catch_all_on_enum`: principle "Exhaustive Matching", show explicit variants
- [ ] `no_unwrap`: principle "Error Handling", show `?` or `match` alternative
- [ ] `no_panic`: principle "Fail Fast Correctly", explain panic vs error
- [ ] `no_recursion`: principle "Simple Control Flow", suggest iteration

## Phase 3: Other domain diagnostics

- [ ] `ambient_clock`: principle "Functional Core", show injection pattern
- [ ] `contradictory_time`: principle "Functional Core", show parameter-only pattern
- [ ] `verified_purity`: principle "Verified Module Purity", show pure alternatives
- [ ] `bool_naming`: principle "Positive Predicates", show prefixed pattern
- [ ] `acronym_style`: principle "Acronym Capitalization", show single-word form
- [ ] `numeric_units`: principle "Unit Suffixes", show suffixed name
- [ ] `float_for_currency`: principle "Integer Currency", show microdollar pattern
- [ ] `function_length`: principle "Function Length", show split guidance
- [ ] `compound_condition`: principle "Condition Decomposition", show nested-if pattern
- [ ] `ambiguous_params`: principle "Explicit Interfaces", show options struct
- [ ] `platform_dependent_cast`: principle "Explicit Types", show fixed-width pattern
- [ ] `unjustified_allow`: principle "Documented Suppression", show comment pattern
- [ ] `nested_conditionals`: principle "Simple Control Flow", show flat alternative
- [ ] `multi_lock_ordering`: principle "Lock Ordering", show comment pattern

## Phase 4: Lint groups

- [ ] Register `tigerstyle::safety` group (12 lints)
- [ ] Register `tigerstyle::naming` group (4 lints)
- [ ] Register `tigerstyle::structure` group (6 lints)
- [ ] Register `tigerstyle::purity` group (3 lints)
- [ ] Register `tigerstyle::concurrency` group (1 lint)

## Phase 5: Fixture updates

- [ ] Update all existing `.stderr` fixtures to match new diagnostic format
- [ ] Verify all fixtures pass in `nix flake check`
