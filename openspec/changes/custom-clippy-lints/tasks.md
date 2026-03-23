# Tasks: Custom Clippy Lints

## Phase 1: Scaffolding

- [ ] Initialize Rust crate with `dylint_linting` dependency and nix flake
- [ ] Set up `ui_tests/` directory with `dylint_testing` harness
- [ ] Create `src/lib.rs` with lint registration skeleton
- [ ] Create module structure: `safety/`, `naming/`, `structure/`, `purity/`
- [ ] Verify `cargo test` runs and passes with zero lints registered

## Phase 2: Tier 1 Lints (syntax-only, Early pass)

- [ ] `compound_assertion` — detect `assert!(a && b)`, suggest split
- [ ] `function_length` — detect functions over 70 non-blank/non-comment lines
- [ ] `acronym_style` — detect `RPCHandler` style, suggest `RpcHandler`
- [ ] `numeric_units` — detect bare `timeout`, `size` without unit suffix

## Phase 3: Tier 1 Lints (type-aware, Late pass)

- [ ] `bool_naming` — detect bool bindings without `is_`/`has_`/`should_`/`can_` prefix
- [ ] `ignored_result` — detect `let _ = fallible_expr()` discarding Result

## Phase 4: Tier 2 Lints

- [ ] `unbounded_loop` — detect `loop {}` without iteration bound
- [ ] `compound_condition` — detect `if` with 3+ boolean operators
- [ ] `verified_purity` — detect async/IO/time in `verified/` modules

## Phase 5: Tier 3 Lints

- [ ] `assertion_density` — warn on 20+ line functions with < 2 assertions
- [ ] `ambiguous_params` — detect 2+ consecutive same-type primitive params

## Phase 6: Integration

- [ ] Run against aspen workspace, triage results
- [ ] Add default severity levels from design doc
- [ ] Write README with usage instructions and lint catalog
- [ ] Add as flake check in aspen's `flake.nix`
- [ ] Configure allowlists for known exceptions in aspen
