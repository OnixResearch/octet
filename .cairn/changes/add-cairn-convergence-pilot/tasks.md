# Tasks: Add the Cairn convergence pilot contract

## Phase 1: Contract intake

- [x] [serial] Import Cairn's `contracts/assurance-targets.ncl` as the exact `cairn-assurance-targets` source, project it to `cairn-policy/assurance-targets.ncl`, and record the notice entry. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Run `tools/import-public-dependencies.py` in `--sync` mode and again in check mode, retaining both transcripts. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Author the proposal, design, tasks, and delta spec and pass `validate` plus the proposal, design, and tasks gates. r[octet.cairn_convergence_pilot.contract]

## Phase 2: Registry evidence

- [x] [serial] Export and verify Octet's requirement registry at pilot revision `d5b34c2d64b1227417a46df8ed8427f2d10ed3c8` and retain both receipts. r[octet.cairn_convergence_pilot.evidence]
- [ ] [serial] Copy the six row fields for `octet.aeneas_consumer_contract.compatibility` from the verified export into `standards/requirements/cairn-pilot/targets.ncl` under one target with `assurance = "checked_proof"` and `evidence_job = { job_id = "octet.aeneas.external-consumer.mantle.v1", definition = "standards/proof-rails/aeneas-lean/mantle-job.ncl" }`. r[octet.cairn_convergence_pilot.contract]
- [ ] [serial] Write `claim` from what the external-consumer theorem states and `informal_remainder` for what the requirement prose asks beyond it. r[octet.cairn_convergence_pilot.contract]

## Phase 3: Generated record and contract

- [ ] [serial] Build `standards/requirements/cairn-pilot/generated.ncl` from the owner-produced envelope's `source_bindings` with all four `required_bindings` lists empty, and export `generated.json`. r[octet.cairn_convergence_pilot.contract]
- [ ] [serial] Apply the vendored `project` to the targets contract and the generated record, export `targets.json` and `contract.json`, and copy the realized `external-receipt.json` into `standards/requirements/fixtures/cairn-pilot/` with its path and BLAKE3 in a README. r[octet.cairn_convergence_pilot.contract]

## Phase 4: Judgment

- [ ] [serial] Write `standards/requirements/fixtures/cairn-pilot/evidence.json` with the pilot `contract_id`, `contract_blake3` over `contract.json`, the realized proof deck with its `evidence_id` and `artifact_root`, and explicit `non_claims`. r[octet.cairn_convergence_pilot.evidence]
- [ ] [serial] Run `cargo octet requirements check` and `cargo octet requirements verify` with `--source-root` at the realized output, retain `decision.json`, and copy it into the fixture directory. r[octet.cairn_convergence_pilot.judgment]

## Phase 5: Family registration and manifest

- [ ] [serial] Register the pilot targets/contract family in `standards/policies/registry.ncl` as Nickel-authored and refresh `octet-artifact-manifest.json` with its generator. r[octet.cairn_convergence_pilot.evidence]
- [ ] [serial] Rerun `controls/standards` freshness checks and the Octet change validation after the registration lands. r[octet.cairn_convergence_pilot.evidence]

## Verification Coverage

- `Scenario: Exact contract intake is byte-identical to the admitted source` -> `tools/import-public-dependencies.py`, `evidence/import-check.txt`
- `Scenario: Hand-edited projection is rejected` -> `tools/import-public-dependencies.py`, `evidence/import-check.txt`
- `Scenario: Registry export is verified at the frozen revision` -> `evidence/registry-export.json`, `evidence/registry-verify.json`
- `Scenario: Export receipt names a moved revision` -> `evidence/registry-export.json`
- `Scenario: Octet decision accepts the realized target` -> follow-on task, `standards/requirements/fixtures/cairn-pilot/decision.json`
- `Scenario: Foreign success does not accept the target` -> follow-on task, `standards/requirements/fixtures/cairn-pilot/decision.json`
