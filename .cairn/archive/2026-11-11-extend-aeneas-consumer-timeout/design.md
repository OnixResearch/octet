# Design: Extend the Aeneas consumer proof budget

## Context

The external-consumer job builds the pinned Mathlib source inside the provider
closure. At 1,800,000 ms the Lean stage reaches 1,549/1,632 modules and is
terminated. The remaining modules need a larger bounded budget on the supported
host.

## Decisions

- Set `configured_timeout_ms = 2700000` in `consumer-profile.ncl`.
- Set `aeneas_timeout_ms = 2700000` in the proof-rail manifest.
- Raise `PROVIDER_TIMEOUT_MAX_MS` and the live/fixture test budgets to
  2,700,000 ms.
- Regenerate `generated/consumer-profile.json`, update its BLAKE3 identity in
  `mantle-job.ncl` and the expected external receipt, and update the fixture
  timeout fields.
- Keep stdout/stderr limits, the environment allowlist, and network denial
  unchanged.

## Verification

- Focused `octet-standards` and `cargo-octet` Aeneas tests pass.
- `mantle-aeneas-consumer-job` completes within the extended budget.
- `nix flake check -L` passes.

## Non-claims

The timeout is a resource bound, not proof evidence. It does not prove Aeneas,
Charon, Lean, or Mathlib correctness, and it does not claim the cold build is
fast or deterministic.
