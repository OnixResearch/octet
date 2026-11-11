# Tasks: Recorded-only Vermilion canary rail

- [x] [serial] Add the closed canary receipt validator and tamper/drift tests. r[octet.vermilion_canary_rail.recorded_only]
- [x] [serial] Add the recorded-only proof rail and advisory verification policy. r[octet.vermilion_canary_rail.recorded_only]
- [x] [serial] Add the `cargo octet prove` canary branch and manual-review proof card. r[octet.vermilion_canary_rail.recorded_only]
- [x] [serial] Preserve the recorded-only tier through artifact verification and Valence export. r[octet.vermilion_canary_rail.recorded_only]
- [x] [serial] Run focused tests, self-check, and the full flake check. r[octet.vermilion_canary_rail.recorded_only]
  - Focused tests, the CLI canary prove/artifact verify/Valence export, the self-check, and the full flake check pass. The object-corpus refresh is tracked in `refresh-reviewed-object-corpus-evidence-canary`; the Aeneas budget extension is tracked in `extend-aeneas-consumer-timeout`. Evidence: `evidence/full-flake-check.json`.

## Verification Coverage

- `Scenario: Measured canary is validated` -> canary validator test and prove integration test
- `Scenario: Canary drift fails closed` -> canary tamper/drift tests
- `Scenario: Tiers survive artifact verification and Valence export` -> recorded artifact bundle verification and Valence export
