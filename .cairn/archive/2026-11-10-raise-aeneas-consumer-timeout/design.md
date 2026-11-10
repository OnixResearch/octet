# Design: Raise the Aeneas consumer proof budget

## Context

The external-consumer profile is a closed Nickel document exported to
`generated/consumer-profile.json`. The Mantle producer reads the generated
profile's timeout, and the expected receipt records the same value. The Lean
stage builds the pinned Mathlib source inside the provider closure; a cold build
exceeds the current 900,000 ms budget on the supported host.

## Decisions

- Set `configured_timeout_ms = 1800000` in `consumer-profile.ncl`.
- Raise `PROVIDER_TIMEOUT_MAX_MS` in the consumer validation and the envelope
  test expectation to 1,800,000 ms so the provider admits the consumer bound.
- Raise `aeneas_timeout_ms` in `standards/proof-rails/manifest.ncl` so the
  selected Aeneas rail and the consumer profile agree on the stage budget.
- Regenerate `generated/consumer-profile.json` with the checked Nickel export.
- Update the profile identity in `mantle-job.ncl` and the expected external
  receipt to the regenerated BLAKE3 identity, and refresh the receipt identity
  that binds the changed consumer resource policy.
- Update the Aeneas fixture verification, proof cards, and receipt to the same
  budget.
- Refresh the advisory self-check baseline because the source edits shift four
  stable identities; keep the 4,262 warning total unchanged.
- Keep stdout/stderr limits, the environment allowlist, and network denial
  unchanged.

## Verification

- `nix build .#checks.x86_64-linux.mantle-aeneas-consumer-job` completes with a
  supported receipt.
- Focused Aeneas/consumer tests pass.
- `nix flake check -L` reports no Aeneas consumer failure.

## Non-claims

The timeout is a resource bound, not proof evidence. The change does not prove
Aeneas, Charon, Lean, or Mathlib correctness, and it does not claim that the
cold build is deterministic or fast.
