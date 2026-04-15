## Phase 1: Diagnostic foundation

- [ ] Create `src/diagnostics.rs` with `LintDiagnosticMeta` inventory and `emit_tiger_style_lint()` wrapper
- [ ] Declare and wire the new `diagnostics` module from `src/lib.rs` so lint modules can compile against the shared wrapper
- [ ] Record principle, guide section, domain group, guidance, and example text policy for all 25 lints in one metadata table
- [ ] Copy the canonical principle, guide section, domain group, guidance, and example text strings verbatim from the design appendix into the metadata inventory
- [ ] Add a unit test proving diagnostic metadata names exactly match `LINT_INVENTORY`
- [ ] Add a full metadata snapshot test covering lint name, principle, guide section, domain group, guidance, example mode, and example text
- [ ] Assert every `ExampleMode::Required` entry has concrete example text
- [ ] Add a shared-helper enforcement test that source-scans lint modules and rejects `Tiger Style (`, `see Tiger Style guide:`, `.help(`, `.note(`, `span_help(`, and `span_note(` outside `src/diagnostics.rs`

## Phase 2: Wire rich diagnostics into lint emissions

- [ ] Route every lint emission through `emit_tiger_style_lint()` and remove direct help/note emission from lint modules before any snapshot refresh
- [ ] Keep explicit lint/pass registration code as the source of truth for pass wiring; do not introduce a runtime function-pointer registration table while adding the shared inventory
- [ ] Bind each lint below to its exact appendix-derived `LintDiagnosticMeta` row; do not paraphrase principle, guide section, domain group, guidance, example mode, or example text during rewiring
- [ ] Safety lints: `compound_assertion`, `assertion_density`, `unbounded_loop`, `ignored_result`, `unchecked_narrowing`, `unchecked_division`, `raw_arithmetic_overflow`, `catch_all_on_enum`, `no_unwrap`, `no_panic`, `no_recursion`
- [ ] Naming lints: `bool_naming`, `acronym_style`, `numeric_units`, `float_for_currency`
- [ ] Structure lints: `function_length`, `compound_condition`, `ambiguous_params`, `platform_dependent_cast`, `unjustified_allow`, `nested_conditionals`
- [ ] Purity lints: `ambient_clock`, `contradictory_time`, `verified_purity`
- [ ] Concurrency lint: `multi_lock_ordering`

## Phase 3: Register and verify domain groups

- [ ] Register domain groups from inventory-derived lists of the original static lint IDs; do not clone `Lint` descriptors or use a runtime registration table
- [ ] Register `tigerstyle::safety` group from the shared metadata inventory (11 lints)
- [ ] Register `tigerstyle::naming` group from the shared metadata inventory (4 lints)
- [ ] Register `tigerstyle::structure` group from the shared metadata inventory (6 lints)
- [ ] Register `tigerstyle::purity` group from the shared metadata inventory (3 lints)
- [ ] Register `tigerstyle::concurrency` group from the shared metadata inventory (1 lint)
- [ ] Add a full group-membership snapshot test covering all five group names and their ordered lint members
- [ ] Add standalone UI fixture proving `#![allow(tigerstyle::safety)]` suppresses safety lints while a different-domain lint still warns
- [ ] Add standalone UI fixture proving `#![deny(tigerstyle::naming)]` escalates a naming lint to an error
- [ ] Keep group fixtures self-contained with `#![feature(register_tool)]`, `#![register_tool(tigerstyle)]`, and the existing `--edition=2024` UI path

## Phase 4: Snapshot and emitted-diagnostic validation

- [ ] Add a deterministic `lint -> emitting fixture` coverage manifest for all 25 lints and test that it matches the metadata inventory
- [ ] Prove every `lint -> emitting fixture` mapping points to a fixture that actually emits that lint and is not a suppressed/non-emitting case
- [ ] Add a deterministic fixture-set artifact listing the full existing UI fixture corpus plus the new group allow/deny fixtures
- [ ] Add a test validating that the fixture-set artifact covers the full UI fixture corpus plus the new group allow/deny fixtures
- [ ] Update all existing UI `.stderr` fixtures plus the new group allow/deny fixture snapshots to the canonical warning/help/note format
- [ ] Run `cargo test --lib`
- [ ] Run `cargo test --test ui -- --ignored --nocapture`
- [ ] Verify all fixtures and snapshots pass in `nix flake check`
