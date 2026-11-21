# Tasks: Derive the pilot target row from the registry export

## Phase 1: Vendor refresh

- [x] [serial] Set `cairn-assurance-targets` to Cairn revision `f20bcb4aa00268fdaa630cbce07633250f18a681` with the archive SHA-256 the tool's own archive command produces for `LICENSE-MIT LICENSE-APACHE contracts/assurance-targets.ncl`, then run `tools/import-public-dependencies.py --sync` and check mode. r[octet.cairn_convergence_pilot.contract]

## Phase 2: Targets cutover

- [x] [serial] Import the pilot registry export, derive `specification` with `specification_from_registry` and each target `requirement` with `requirement_from_registry`, delete the copied fields, and keep every other target field unchanged. r[octet.cairn_convergence_pilot.evidence]
- [x] [serial] Regenerate `targets.json`, `contract.json`, and `generated.json` with their documented commands and require byte identity with the committed files. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Rerun the pilot `requirements check` and `requirements verify` with `--source-root /home/brittonr/wt/octet-rev` and require the committed decision digest. r[octet.cairn_convergence_pilot.judgment]

## Phase 3: Negative and freshness evidence

- [x] [serial] Export a temporary targets copy naming an absent requirement id and a temporary export copy repeating the pilot row, and save the helper failures as evidence. r[octet.cairn_convergence_pilot.evidence]
- [x] [serial] Run the vendor check, `octet-policy-validate`, the policy artifact refresh, `policy-registry`, `standards-fixtures`, `cargo test -p octet-standards`, and the flake `core` check once, and save the transcripts. r[octet.cairn_convergence_pilot.contract]

## Verification Coverage

- `Scenario: Exact contract intake is byte-identical to the admitted source` -> `tools/import-public-dependencies.py`, `evidence/import-check.txt`
- `Scenario: Registry export is verified at the frozen revision` -> `evidence/standards-checks.txt`
- `Scenario: Derived row reproduces the verified export row` -> `evidence/standards-checks.txt`, `evidence/requirements-check.txt`
- `Scenario: Missing or duplicated export row is rejected` -> `evidence/negative-unknown-id.txt`, `evidence/negative-duplicate-row.txt`
- `Scenario: Octet decision accepts the realized target` -> `evidence/requirements-check.txt`, `evidence/requirements-verify.txt`
