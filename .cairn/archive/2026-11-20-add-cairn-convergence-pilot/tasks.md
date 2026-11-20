# Tasks: Add the Cairn convergence pilot contract

## Phase 1: Contract intake

- [x] [serial] Import Cairn's `contracts/assurance-targets.ncl` as the exact `cairn-assurance-targets` source, project it to `cairn-policy/assurance-targets.ncl`, and record the notice entry. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Run `tools/import-public-dependencies.py` in `--sync` mode and again in check mode, retaining both transcripts. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Author the proposal, design, tasks, and delta spec and pass `validate` plus the proposal, design, and tasks gates. r[octet.cairn_convergence_pilot.contract]

## Phase 2: Registry evidence

- [x] [serial] Export and verify Octet's requirement registry at pilot revision `d5b34c2d64b1227417a46df8ed8427f2d10ed3c8` and retain both receipts. r[octet.cairn_convergence_pilot.evidence]
- [x] [serial] Copy the six row fields for `octet.aeneas_consumer_contract.compatibility` from the verified export into `standards/requirements/cairn-pilot/targets.ncl` under one target with `assurance = "checked_proof"` and `evidence_job = { job_id = "octet.aeneas.external-consumer.mantle.v1", definition = "standards/proof-rails/aeneas-lean/mantle-job.ncl" }`. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Write `claim` from what the external-consumer theorem states and `informal_remainder` for what the requirement prose asks beyond it. r[octet.cairn_convergence_pilot.contract]

## Phase 3: Generated record and contract

- [x] [serial] Build `standards/requirements/cairn-pilot/generated.ncl` from the owner-produced envelope's `source_bindings` with all four `required_bindings` lists empty, add one `specifications` entry holding the plain BLAKE3 of `.cairn/specs/aeneas-consumer-contract/spec.md` at the pilot revision, and export `generated.json`. r[octet.cairn_convergence_pilot.contract]
- [x] [serial] Apply the vendored `project` to the targets contract and the generated record, export `targets.json` and `contract.json`, byte-copy the realized `external-receipt.json` into `standards/requirements/fixtures/cairn-pilot/`, and record its source path and plain BLAKE3 `84d2e77717013a203a4daf6ce649de8e501c7386f142b7abc66f32d08c33420e` in `standards/requirements/cairn-pilot/README.md`. r[octet.cairn_convergence_pilot.contract]

## Phase 4: Judgment

- [x] [serial] Write `standards/requirements/fixtures/cairn-pilot/evidence.json` with the pilot `contract_id`, `contract_blake3` over `contract.json`, the realized proof deck with its `evidence_id` and `artifact_root`, and explicit `non_claims`. r[octet.cairn_convergence_pilot.evidence]
- [x] [serial] Run `requirements check` and `requirements verify` with `--source-root /home/brittonr/wt/octet-rev`, a clean checkout at REV `d5b34c2d`. The reviewed specification and the source-binding locators are repo-relative at REV, and the proof deck and `artifact_root` in `evidence.json` are absolute paths inside the realized output `/datapool/mantle-store/convergence-pilot-20261119/store/zjq6whhf2c641vkz89kgwwpbajjq5cbq-octet-mantle-aeneas-consumer-job-v1`. Retain `decision.json` and copy it into the fixture directory. r[octet.cairn_convergence_pilot.judgment]

## Phase 5: Family registration and manifest

- [x] [serial] Register the pilot targets/contract family in `standards/policies/registry.ncl` as Nickel-authored. The policy artifacts, including `octet-artifact-manifest.json`, are generated at build time and are not checked in; refresh them with `nix run .#octet-policy-artifacts -- <tmpdir>` and record the generated `octet.cairn-pilot-requirement-contract` family entry. r[octet.cairn_convergence_pilot.evidence]
- [x] [serial] Rerun the controls/standards freshness checks — `policy-registry`, `standards-fixtures`, and `cargo test -p octet-standards` — and the Octet change validation after the registration lands. r[octet.cairn_convergence_pilot.evidence]

## Verification Coverage

- `Scenario: Exact contract intake is byte-identical to the admitted source` -> `tools/import-public-dependencies.py`, `evidence/import-check.txt`
- `Scenario: Hand-edited projection is rejected` -> `tools/import-public-dependencies.py`, `evidence/import-check.txt`
- `Scenario: Registry export is verified at the frozen revision` -> `evidence/registry-export.json`, `evidence/registry-verify.json`
- `Scenario: Export receipt names a moved revision` -> `evidence/registry-export.json`
- `Scenario: Octet decision accepts the realized target` -> `evidence/requirements-check.txt`, `evidence/requirements-verify.txt`, `standards/requirements/fixtures/cairn-pilot/decision.json`
- `Scenario: Foreign success does not accept the target` -> `evidence/requirements-verify.txt`, `standards/requirements/fixtures/cairn-pilot/decision.json`
