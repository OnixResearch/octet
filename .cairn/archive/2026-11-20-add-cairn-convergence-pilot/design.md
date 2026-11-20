# Design: Add the Cairn convergence pilot contract

## Context

Octet vendors exact upstream source selections through
`vendor/dependencies.ncl` and `tools/import-public-dependencies.py`. The existing
`cairn` source imports policy inputs but not `contracts/`, so Cairn's
consumer-facing assurance-targets contract has no exact intake. Cairn's
convergence command reads five artifacts per targeted row: the requirement
registry, the targets file, the checked-in Octet contract, the Octet decision,
and the Mantle records for the named evidence job. Only the registry is
available before the producer output exists.

## Decisions

### Decision: Import the contract as a separate exact source

**Choice:** Add `cairn-assurance-targets` at revision
`3dc654d38404d595ff07ca994925789f45941ffe` selecting `LICENSE-MIT`,
`LICENSE-APACHE`, and `contracts/assurance-targets.ncl`, projected to
`cairn-policy/assurance-targets.ncl`, with archive SHA256
`a39ebb184e8fc94db83604d14b227fd1a1b0d7985d29a6ac5dd1902c96578f74` and file
BLAKE3 `747ae43ec2fe64351dbe2c8e012550f1f8ae12c4a686d59dedb1b41ae3f0d35a`.

**Rationale:** The projection is one byte-identical file, so the contract is
refreshed by the repo-owned tool and bound by `vendor/SHA256SUMS` instead of by
review attention. Reusing the existing `cairn` source would either widen that
source's selection for an unrelated purpose or make the projection depend on a
revision whose policy inputs Octet already pins.

### Decision: Freeze the registry at the pilot revision

**Choice:** Export the registry from a detached worktree at
`d5b34c2d64b1227417a46df8ed8427f2d10ed3c8` and retain the export and verify
receipts with the change.

**Rationale:** A target binds one content-bound row, including the row's
`row_blake3` and the specification identity. Exporting at the worktree's own
HEAD would silently rebind the row on every later commit.

### Decision: Take the reviewed specification digest from the generated record

**Choice:** The generated source record carries `specifications: [{ path,
blake3 }]` holding the plain BLAKE3 of the reviewed specification file, and
`project` reads `reviewed_specification` from that entry instead of from the
target's contract.

**Rationale:** Octet verifies the plain BLAKE3 of the specification bytes,
while Cairn's registry stores a domain-separated specification hash. Taking the
digest from the tool-produced record keeps the two identities from being
conflated and keeps the plain digest out of hand-authored target text.

### Decision: Keep the pilot steps separate from this package

**Choice:** Ship the intake and registry evidence here; leave the targets,
generated record, contract, evidence bundle, decision, family registration,
and manifest refresh as unchecked follow-on tasks.

**Rationale:** Those artifacts bind a realized producer output that does not
exist yet. Writing them early would encode an unrealized claim.

## Risks / Trade-offs

- The pinned Cairn revision is a local validation candidate, not a released
  package; the notice entry says so.
- The projected contract is inert until the targets file exists; nothing reads
  it yet, so a contract-only import cannot change judgment behavior.

## Verification

- `tools/import-public-dependencies.py --sync` followed by check mode passes.
- `nix --option min-free 0 run .#cairn -- validate --root .` passes.
- `gate proposal`, `gate design`, and `gate tasks` pass for
  `add-cairn-convergence-pilot`.

## Non-claims

The import is typed contract intake, not proof evidence. It does not prove the
external-consumer theorem, admit any producer output, or claim Cairn lifecycle
approval or release qualification.
