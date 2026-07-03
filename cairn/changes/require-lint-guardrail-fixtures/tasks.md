## Contract

- [ ] [serial] Specify the non-emitting guardrail fixture manifest schema and allowed scenario categories. r[octet.lint_guardrail_fixtures.manifest]
- [ ] [serial] Specify that every registered lint must have at least one emitting fixture and one guardrail fixture. r[octet.lint_guardrail_fixtures.complete]

## Implementation

- [ ] [serial] Add `tests/fixtures/lint_guardrail_fixtures.json` with entries for all registered lints. r[octet.lint_guardrail_fixtures.manifest]
- [ ] [serial] Extend `src/lib.rs` fixture freshness tests to reject missing guardrails, stale fixture names, and unknown lint names. r[octet.lint_guardrail_fixtures.complete]
- [ ] [parallel] Add missing `ui_tests/` guardrail fixtures for lints that currently only have emitting coverage. r[octet.lint_guardrail_fixtures.complete]

## Verification

- [ ] [serial] Run `nix develop -c bash -c 'cargo test --lib lint_guardrail'` or the closest focused fixture-manifest test. r[octet.lint_guardrail_fixtures.manifest]
- [ ] [serial] Run `nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'`. r[octet.lint_guardrail_fixtures.complete]
- [ ] [serial] Run Cairn validation/gates. r[octet.lint_guardrail_fixtures.complete]
