# Cairn Convergence Pilot Specification

## Purpose

Defines the `cairn-convergence-pilot` capability.

## Requirements

### Requirement: Exact assurance-targets intake

r[octet.cairn_convergence_pilot.contract] Octet MUST admit Cairn's
consumer-facing assurance-targets contract as an exact vendored source that
records the repository, the admitted revision, the selected license and
contract paths, the archive SHA256, the projected file BLAKE3, and one
projection to `cairn-policy/assurance-targets.ncl`, and it MUST refresh that
copy only through `tools/import-public-dependencies.py` so the checked-in
projection stays byte-identical to the admitted Git archive selection.

#### Scenario: Exact contract intake is byte-identical to the admitted source

- GIVEN the admitted Cairn revision and its `contracts/assurance-targets.ncl`
- WHEN dependency sync and check mode run
- THEN the projected `cairn-policy/assurance-targets.ncl` MUST equal the source
  bytes and its recorded digest MUST match `vendor/SHA256SUMS`
- AND the recorded archive SHA256 MUST match the tool's own archive command.

#### Scenario: Hand-edited projection is rejected

- GIVEN a projected contract file edited without a new admitted revision
- WHEN dependency check mode runs
- THEN the check MUST fail with a projection or checksum drift error.

### Requirement: Content-bound registry evidence

r[octet.cairn_convergence_pilot.evidence] The pilot MUST export Octet's
requirement registry at the frozen pilot revision with Cairn's
`requirement-registry export`, retain the export and verify receipts with the
change, and MUST derive the six row fields and the contract specification of
the targeted requirement from that export with Cairn's
`requirement_from_registry` and `specification_from_registry` helpers instead
of authoring them.

#### Scenario: Registry export is verified at the frozen revision

- GIVEN a detached worktree at the frozen pilot revision
- WHEN `requirement-registry export` and `requirement-registry verify` run
- THEN both receipts MUST bind the repository id, the Git revision kind, and
  the exact pilot revision
- AND the export MUST contain the targeted requirement row.

#### Scenario: Export receipt names a moved revision

- GIVEN an export produced without an explicit revision at a worktree whose
  HEAD moved
- WHEN the receipt is compared with the frozen pilot revision
- THEN the receipt MUST NOT be accepted as pilot evidence.

#### Scenario: Derived row reproduces the exported row

- GIVEN the retained pilot export and the vendored Cairn contract helpers
- WHEN the pilot targets and contract are exported
- THEN the derived row and specification MUST equal the values the export holds
  for the targeted requirement id
- AND the regenerated targets, contract, and generated record MUST be
  byte-identical to the committed files.

#### Scenario: Missing or duplicated export row is rejected

- GIVEN a targets file naming a requirement id the export does not contain, or
  an export that holds the targeted row twice
- WHEN the targets file is exported
- THEN the export MUST fail with the helper's
  "must contain exactly one row" message.

### Requirement: Octet judgment is the acceptance authority

r[octet.cairn_convergence_pilot.judgment] The pilot MUST derive its decision
from Octet's own `requirements check` over the reviewed pilot contract and the
realized output's evidence bundle, MUST confirm it with `requirements verify`,
and MUST retain that decision as the only acceptance record; a foreign
provider success, a cache hit, or a relayed outcome alone MUST NOT accept the
target.

#### Scenario: Octet decision accepts the realized target

- GIVEN the reviewed pilot contract and an evidence bundle whose identities
  match the realized output
- WHEN `requirements check` and `requirements verify` run with the realized
  output as source root
- THEN check MUST exit successfully with a passing aggregate
- AND verify MUST recompute and report `passed`.

#### Scenario: Foreign success does not accept the target

- GIVEN a realized output whose producer reports success but whose decision
  record is missing, unbound, or rejected
- WHEN the pilot row is judged
- THEN the target MUST remain unaccepted regardless of the producer outcome.
