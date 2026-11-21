# Proposal: Derive the pilot target row from the registry export

## Why

Cairn revision `f20bcb4aa00268fdaa630cbce07633250f18a681` adds
`requirement_from_registry` and `specification_from_registry` to
`contracts/assurance-targets.ncl`. A consumer can now select a target's six row
fields and its contract `specification` by requirement id from a checked-in
`cairn requirement-registry export`. Octet's pilot targets file still
hand-copies those values, so a later export refresh can silently disagree with
the authored target.

## What Changes

- Re-vendor the `cairn-assurance-targets` source at the new revision, archive
  SHA-256 `7fda0d723d5d29434330b8018b36c36dca220744883924bb005f25a929326e5b`,
  through `tools/import-public-dependencies.py` only.
- Rewrite `standards/requirements/cairn-pilot/targets.ncl` so `specification`
  and the target `requirement` come from
  `specification_from_registry`/`requirement_from_registry` over
  `../fixtures/cairn-pilot/requirement-registry.json`.
- Keep every other target field, `targets.json`, `contract.json`,
  `generated.json`, and the pilot decision byte-identical.
- Record the negative cases: an unknown requirement id and a duplicated export
  row must fail with the helper's "exactly one row" message.

## Impact

- **Standards**: `standards/requirements/cairn-pilot/targets.ncl`.
- **Vendoring**: `vendor/dependencies.ncl`, `vendor/dependencies.json`,
  `vendor/SHA256SUMS`, `vendor/sources/cairn-assurance-targets/`,
  `cairn-policy/assurance-targets.ncl`.
- **Lifecycle**: this change package and its `evidence/` receipts.

## Non-goals

- No Rust, schema, producer job, profile, or theorem-policy change.
- No change to the exported target contract, its decision, or its evidence.
- No new fixture tree; the negative checks stay uncommitted transcripts.
