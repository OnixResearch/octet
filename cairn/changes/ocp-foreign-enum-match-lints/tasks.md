## Contract

- [ ] [serial] Define the `foreign_enum_catch_all` lint contract: flag `_` match arms over foreign-crate enums, default `allow`. r[octet.ocp_enum_match.foreign_catch_all]
- [ ] [serial] Define the `fragile_exhaustive_enum_match` lint contract: flag exhaustive matches over same-crate enums without a `_` arm, default `allow`, honor a sealed-enum marker. r[octet.ocp_enum_match.fragile_exhaustive]
- [ ] [serial] Define the `sealed_enum_marker` config key (default `octet::sealed_enum`) that suppresses the fragile-match lint per enum. r[octet.ocp_enum_match.sealed_marker]

## Implementation

- [ ] [serial] Implement `foreign_enum_catch_all` under `src/safety/` reusing `catch_all_on_enum` arm detection plus crate-of-origin resolution. r[octet.ocp_enum_match.foreign_catch_all]
- [ ] [serial] Implement `fragile_exhaustive_enum_match` under `src/safety/` with the heuristic no-`_`-arm-plus-all-variants detection. r[octet.ocp_enum_match.fragile_exhaustive]
- [ ] [serial] Add the `sealed_enum_marker` config key and honor the marker attribute in the fragile-match lint. r[octet.ocp_enum_match.sealed_marker]
- [ ] [serial] Register both lints in `src/lib.rs`, add diagnostic metadata, update `docs/lint-reference.md` and `templates/dylint.toml` catalog counts. r[octet.ocp_enum_match.foreign_catch_all]

## Verification

- [ ] [serial] Add positive and negative UI fixtures for `foreign_enum_catch_all`. r[octet.ocp_enum_match.foreign_catch_all]
- [ ] [serial] Add positive and negative UI fixtures for `fragile_exhaustive_enum_match`. r[octet.ocp_enum_match.fragile_exhaustive]
- [ ] [serial] Add a cross-crate fixture proving `foreign_enum_catch_all` only fires on foreign enums and skips macro-generated scrutinees. r[octet.ocp_enum_match.foreign_catch_all]
- [ ] [serial] Add an opt-in marker fixture proving the fragile-match diagnostic is suppressed for `#[octet::sealed_enum]`. r[octet.ocp_enum_match.sealed_marker]
- [ ] [serial] Update `tests/fixtures/ui_fixture_set.json` and `tests/fixtures/lint_emitting_fixtures.json` for both lints. r[octet.ocp_enum_match.foreign_catch_all]
- [ ] [serial] Run the UI rail, the autofix rail, `cargo test --lib`, and Cairn gates. r[octet.ocp_enum_match.foreign_catch_all]
