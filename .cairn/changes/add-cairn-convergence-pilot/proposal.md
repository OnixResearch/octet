# Proposal: Add the Cairn convergence pilot contract

## Why

Cairn now owns a consumer-facing assurance-targets contract and a read-only
`convergence inspect` command that judges whether Octet's requirement targets
are current, realized, and accepted. Octet has the pieces that command reads —
a requirement registry, reviewed requirement contracts, and Octet decisions —
but no reviewed package that binds them, and no exact intake of the contract
shapes Cairn uses to admit a targets file.

The `octet.aeneas_consumer_contract.compatibility` row is the requirement the
external-consumer profile binds its theorem to. It is the pilot row: Octet
declares one target for it, notes what the theorem actually states, and keeps
the rest of the prose visible as an informal remainder.

## What Changes

- Import Cairn's `contracts/assurance-targets.ncl` as new exact vendored source
  `cairn-assurance-targets` at Cairn revision
  `585f50d90be646742167c862a8c7d5d08f41e152`, projected to
  `cairn-policy/assurance-targets.ncl` by the repo-owned import tool only.
- Export Octet's requirement registry at the frozen pilot revision
  `d5b34c2d64b1227417a46df8ed8427f2d10ed3c8` with Cairn's
  `requirement-registry export`, verify it, and retain both receipts.
- Add three requirements for the pilot: the vendored contract intake, the
  registry row the target is copied from, and Octet's judgment as the only
  acceptance authority.
- Add the follow-on tasks for the pilot targets, generated source record,
  Octet contract, evidence bundle, `requirements check`/`verify` decision,
  policy-family registration, and artifact-manifest refresh.

## Impact

- **Standards**: `cairn-policy/assurance-targets.ncl`;
  `standards/requirements/cairn-pilot/` and
  `standards/requirements/fixtures/cairn-pilot/` (follow-on tasks).
- **Vendoring**: `vendor/dependencies.ncl`, `vendor/dependencies.json`,
  `vendor/SHA256SUMS`, the new `vendor/sources/cairn-assurance-targets/`
  selection, and the `THIRD_PARTY_NOTICES.md` entry.
- **Lifecycle**: this change package and its `evidence/` receipts.
- **Testing**: import-tool check mode, `cairn validate`, and the proposal,
  design, and tasks gates.

## Non-goals

- No Rust, no Octet schema, no producer job, no profile, and no theorem-policy
  change.
- No claim that the external-consumer theorem proves the requirement prose.
- No realized producer output is required for this package; the targets,
  contract, evidence, and decision steps stay unchecked until that output
  exists.
