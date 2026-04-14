## Phase 1: Config schema extension

- [ ] Add `disabled_lints: Vec<String>` to `Config` (default empty)
- [ ] Add `lint_levels: HashMap<String, String>` to `Config` (default empty)
- [ ] Add validation: warn on unknown lint names at load time
- [ ] Unit tests for config parsing with new fields
- [ ] Unit tests for backward compatibility (old dylint.toml still loads)

## Phase 2: Registration gating

- [ ] Refactor `register_lints` to check `disabled_lints` before each registration
- [ ] Implement lint-level override during registration
- [ ] Integration test: disabled lint produces no diagnostics
- [ ] Integration test: deny-level lint produces error instead of warning

## Phase 3: Templates and docs

- [ ] Update `templates/dylint.toml` with commented `disabled_lints` and `lint_levels` examples
- [ ] Update `docs/downstream-rollout.md` to reference per-lint config for phased rollout
- [ ] Show how starter/boundary/inventory profiles map to config

## Phase 4: Validation

- [ ] Verify config changes work in `nix flake check`
- [ ] Test against Aspen with inventory-first lints disabled via config (confirm clean run)
