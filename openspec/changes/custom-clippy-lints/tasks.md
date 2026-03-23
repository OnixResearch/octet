# Tasks: Custom Clippy Lints

## Phase 1: Scaffolding

- [x] Initialize Rust crate with `dylint_linting` dependency and nix flake ✅ 45m (started: 2026-03-23T00:44Z → completed: 2026-03-23T01:29Z)
- [x] Set up `ui_tests/` directory with `dylint_testing` harness ✅ 10m (started: 2026-03-23T00:50Z → completed: 2026-03-23T01:00Z)
- [x] Create `src/lib.rs` with lint registration skeleton ✅ 5m (started: 2026-03-23T00:48Z → completed: 2026-03-23T00:53Z)
- [x] Create module structure: `safety/`, `naming/`, `structure/`, `purity/`, `concurrency/` ✅ 5m (started: 2026-03-23T00:50Z → completed: 2026-03-23T00:55Z)
- [x] Verify `cargo test` runs and passes with zero lints registered ✅ 5m (started: 2026-03-23T01:25Z → completed: 2026-03-23T01:30Z)

## Phase 2: Tier 1 Lints (syntax-only, Early pass)

- [x] `compound_assertion` — detect `assert!(a && b)`, suggest split ✅ 20m (started: 2026-03-23T01:35Z → completed: 2026-03-23T01:55Z)
- [x] `function_length` — detect functions over 70 non-blank/non-comment lines ✅ 15m (started: 2026-03-23T01:35Z → completed: 2026-03-23T01:50Z)
- [x] `acronym_style` — detect `RPCHandler` style, suggest `RpcHandler` ✅ 15m (started: 2026-03-23T01:35Z → completed: 2026-03-23T01:50Z)
- [x] `numeric_units` — detect bare `timeout`, `size` without unit suffix ✅ 10m (started: 2026-03-23T01:40Z → completed: 2026-03-23T01:50Z)

## Phase 3: Tier 1 Lints (type-aware, Late pass)

- [ ] `bool_naming` — detect bool bindings without `is_`/`has_`/`should_`/`can_` prefix
- [ ] `ignored_result` — detect `let _ = fallible_expr()` discarding Result

## Phase 4: Tier 2 Lints (verified-logic patterns, Late pass)

- [ ] `unchecked_narrowing` — detect bare `as u32`/`u16`/`u8` without bounds check
- [ ] `unchecked_division` — detect `a / b` where `b` is param without zero guard
- [ ] `contradictory_time` — detect time param + `Instant::now()` in same function

## Phase 5: Tier 2 Lints (structure)

- [ ] `unbounded_loop` — detect `loop {}` without iteration bound
- [ ] `compound_condition` — detect `if` with 3+ boolean operators
- [ ] `verified_purity` — detect async/IO/time in `verified/` modules
- [ ] `unjustified_allow` — detect `#[allow(clippy::...)]` without comment

## Phase 6: Tier 2 Lints (clankers patterns)

- [ ] `catch_all_on_enum` — detect `_ => false`/`_ => None` catch-all on enum match
- [ ] `platform_dependent_cast` — detect `as usize`/`as isize` from fixed-width types
- [ ] `float_for_currency` — detect f64/f32 in cost/price/budget struct fields

## Phase 7: Tier 3 Lints

- [ ] `assertion_density` — warn on 20+ line functions with < 2 assertions
- [ ] `ambiguous_params` — detect 2+ consecutive same-type primitive params
- [ ] `raw_arithmetic_overflow` — detect unprotected `+`/`-`/`*` on unsigned types
- [ ] `multi_lock_ordering` — detect structs with 2+ Mutex/RwLock without ordering comment

## Phase 8: Integration

- [ ] Run against aspen workspace, triage results
- [ ] Run against verified-logic, confirm zero false positives on verified code
- [ ] Run against clankers workspace, triage results
- [ ] Add default severity levels from design doc
- [ ] Write README with usage instructions and lint catalog
- [ ] Add as flake check in aspen's `flake.nix`
- [ ] Configure allowlists for known exceptions in each project
