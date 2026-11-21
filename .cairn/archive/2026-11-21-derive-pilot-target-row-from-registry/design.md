# Design: Derive the pilot target row from the registry export

## Context

`standards/requirements/cairn-pilot/targets.ncl` binds exactly one target for
`octet.aeneas_consumer_contract.compatibility`. Until now the six row fields
and the contract `specification` were copied from
`standards/requirements/fixtures/cairn-pilot/requirement-registry.json`, which
is itself a retained `cairn requirement-registry export`. Cairn's new helpers
select exactly one row from such an export by requirement id and fail closed
otherwise, so the copy can be replaced by derivation without changing any
emitted value.

## Decisions

### Decision: Derive both the row and the specification from the export

**Choice:** `specification = Cairn.specification_from_registry registry
"octet.aeneas_consumer_contract.compatibility"` and
`requirement = Cairn.requirement_from_registry registry
"octet.aeneas_consumer_contract.compatibility"`, with `registry` imported from
`../fixtures/cairn-pilot/requirement-registry.json`.

**Rationale:** Both helpers call the same `row_for`, so the row and the
specification can never come from different rows, and the export is already
the retained pilot evidence. The projection still checks the resulting
`specification` against the row and against the generated source record, so
derivation removes the copy without removing a check.

### Decision: Refresh the vendored contract through the import tool only

**Choice:** Bump `cairn-assurance-targets` to
`f20bcb4aa00268fdaa630cbce07633250f18a681` with archive SHA-256
`7fda0d723d5d29434330b8018b36c36dca220744883924bb005f25a929326e5b`, computed
by the tool's own `git archive` selection over
`LICENSE-MIT LICENSE-APACHE contracts/assurance-targets.ncl`, then run sync and
check mode.

**Rationale:** The projection is one byte-identical file bound by
`vendor/SHA256SUMS`; the helpers must arrive through the admitted archive, not
a hand edit.

### Decision: Prove the byte identity, not just the parsed equality

**Choice:** Regenerate `targets.json`, `contract.json`, and `generated.json`
with their documented commands and require `cmp` equality with the committed
bytes; rerun `requirements check` and `requirements verify` against the same
realized output and the clean `d5b34c2d` checkout and require the same decision
digest.

**Rationale:** The cutover is only safe if the derivation reproduces the
copied values exactly; a parsed-JSON comparison could hide key-order or
formatting drift that would change consumer identities.

### Decision: Keep negative checks as transcripts

**Choice:** Run the missing-id and duplicated-row cases from temporary copies
outside the tree and record the helper messages under `evidence/`.

**Rationale:** The helper behavior belongs to the vendored Cairn contract, not
to an Octet fixture, and the `octet.cairn-pilot-requirement-contract` family
declares no negative fixtures.
