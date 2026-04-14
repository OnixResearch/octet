## Phase 1: Config schema extension

- [x] Add `disabled_lints: Vec<String>` to `Config` with `#[serde(default)]` empty default
- [ ] Add `lint_levels: HashMap<String, String>` to `Config` with `#[serde(default)]` empty default
- [ ] Normalize validated `lint_levels` values into an internal representation used by registration
- [ ] Add validation: warn on unknown lint names at load time
- [ ] Add validation: warn on invalid `lint_levels` values and ignore the override
- [ ] Add validation: warn when a lint appears in both `disabled_lints` and `lint_levels`, then ignore the level override
- [ ] Introduce a shared `LINT_INVENTORY` (or equivalent canonical table) that stores every canonical lint name and is the sole source for both config validation and registration wiring
- [ ] Eliminate out-of-band lint registration wiring so every shipped lint is declared through `LINT_INVENTORY` and registration iterates only that inventory
- [ ] Refactor config validation and registration to derive accepted lint names from that shared inventory rather than separate hand-maintained string lists
- [ ] Deterministic check: unit test asserts the shared inventory's lint names are unique and that the lint-name slice passed to config validation is derived from the same inventory used by registration
- [ ] Deterministic check: snapshot-test the exact canonical lint-name set exported by `LINT_INVENTORY` so adding or removing a shipped lint requires updating the inventory snapshot in the same change
- [ ] Unit test: omitted `disabled_lints` and `lint_levels` deserialize to empty defaults
- [ ] Unit test: explicit `disabled_lints` and `lint_levels` entries deserialize successfully
- [ ] Unit test: unknown `disabled_lints` name warns without failing config load
- [ ] Unit test: mixed `disabled_lints` config ignores an unknown name while a separate valid disabled lint still takes effect
- [ ] Unit test: unknown `lint_levels` entry warns without failing config load
- [ ] Unit test: mixed `lint_levels` config ignores an unknown name while a separate valid override still takes effect
- [ ] Unit test: invalid `lint_levels` value warns and keeps the default lint level
- [ ] Unit test: mixed `lint_levels` config ignores an invalid value while a separate valid override still takes effect
- [ ] Unit test: overlap between `disabled_lints` and `lint_levels` warns and keeps the lint disabled

## Phase 2: Registration gating

- [x] Refactor `register_lints` to check `disabled_lints` before each registration
- [ ] Implement lint-level override during registration
- [x] Integration test: disabled lint produces no diagnostics
- [ ] Integration test: disabled lint stays inert even when `lint_levels` tries to raise it
- [ ] Integration test: disabled lint stays inert even when source-level attrs try to raise it
- [x] Fixture-local UI test with local `Cargo.toml` (including empty `[workspace]`), `dylint.toml`, and `src/lib.rs`, using unscoped lint names, proving consumer-owned config is discovered from the fixture root
- [x] Keep UI/config verification on the repo custom UI harness / `checks.ui` path; do not rely on `dylint_testing` or rustup-only UI paths
- [ ] Integration test: `allow` level suppresses diagnostics
- [ ] Integration test: explicit `warn` level emits a warning
- [ ] Integration test: default lint level still emits a warning when no override is present
- [ ] Deterministic check: snapshot-test each `LINT_INVENTORY` entry's default severity so the no-override path preserves shipped defaults across the full inventory
- [ ] Integration test: `deny` level produces an error instead of a warning

## Phase 3: Templates and docs

- [ ] Update `templates/dylint.toml` with commented `disabled_lints` and `lint_levels` examples
- [ ] Document that omitted `disabled_lints` / `lint_levels` keys default to empty collections
- [ ] Document that unknown lint names and invalid level values warn and are ignored
- [ ] Document that `disabled_lints` takes precedence over source-level lint attributes
- [ ] Update `docs/downstream-rollout.md` to reference per-lint config for phased rollout
- [ ] Show how the `docs/downstream-rollout.md` smoke, boundary-first, and inventory-only profiles map to config examples

## Phase 4: Validation

- [ ] Verify config changes in `nix flake check` and the targeted UI fixture gate, after staging any newly added UI fixture paths so the flake source snapshot includes them
- [ ] Human validation: reconstruct Aspen phase 1 from `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` at Aspen commit `909e5e9fa`, use wrapper evidence from `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt` together with `docs/downstream-rollout.md` section `Check-path recipe for nonstandard toolchains` to copy `templates/tigerstyle-check.sh` into that sandbox, apply the inventory-only profile from the same doc, and confirm a clean run
