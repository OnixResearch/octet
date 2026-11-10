# Design: Refresh the self-check warning budget

## Context

The self-check baseline stores stable finding identities for the workspace's
advisory lint set. It is intentionally regenerated when reviewed source changes
alter the finding set. The current baseline was stale after the integrated
changes, so the self-check reported every accepted identity as stale and every
current finding as new.

## Decisions

- Regenerate the baseline with the same packaged `cargo-octet` binary and
  `--write-baseline` path the self-check uses.
- Update `accepted_warning_total` to 4,262 and `accepted_identity_total` to
  3,107 in `flake.nix`.
- Keep the baseline schema, finding schema, metadata, and stable identities
  unchanged in shape.
- Rename the two local boolean identifiers (`has_no_std` to `has_std`, and
  `no_default_features` to `default_features_disabled`) so the baseline does not
  admit findings the self-check validation explicitly excludes; the conditions
  remain equivalent.
- Align the self-check structured and drift assertions with the integrated JSON
  result (`lint_findings`) instead of JSONL line counts and greps, and accept the
  current child-failure diagnostic message while retaining the exit-code and
  sentinel-marker checks.
- Do not add a new suppression or lint-level change; the baseline remains a
  reviewable advisory budget.

## Verification

- `cargo-octet check --baseline standards/self-check-warning-budget/octet-baseline.json`
  reports `accepted_current=3107`, `new=0`, `stale=0`, and 4,262 warnings.
- `nix build .#checks.x86_64-linux.cargo-octet-self-check` passes.
- The full `nix flake check -L` is rerun.

## Non-claims

The refresh does not prove the warnings are harmless, does not make them
release gates, and does not change lint severity or suppression policy.
