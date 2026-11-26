# Tasks: Keep raw prove logs out of the bundle with a build-log retention mode

## Phase 1: Prove retention mode

- [x] [serial] Add `RawLogRetention` and `--raw-log-retention bundle|build-log` parsing, rejecting the option outside proof-rail runs and keeping `bundle` as the default. r[octet.prove_raw_log_retention.retention]
- [x] [serial] In `build-log` mode emit framed raw stage output to stderr, write no log artifacts and no manifest, record the mode and non-claim in `status.json` and `summary.txt`, and keep every in-memory capture check authoritative. r[octet.prove_raw_log_retention.retention]
- [x] [serial] Replace receipt-bound raw log identities with stable build-log markers in `build-log` mode so no bundle file hashes the raw text. r[octet.prove_raw_log_retention.retention]

## Phase 2: Verification and consumer job

- [x] [serial] Parse `raw_log_retention` in `artifact verify`, accept log-free build-log prove bundles, and reject build-log bundles carrying raw logs or a manifest without changing bundle-mode checks. r[octet.prove_raw_log_retention.verification]
- [x] [serial] Pass `--raw-log-retention build-log` in `nix/mantle-aeneas-job.nix` and update its failure-path comments and output to point at the build log. r[octet.prove_raw_log_retention.consumer_job]
- [x] [serial] Update the prove help, the Aeneas consumer/Mantle documentation and the changelog. r[octet.prove_raw_log_retention.retention]

## Phase 3: Tests and evidence

- [x] [serial] Add fake-Aeneas tests for no bundle logs/manifest plus recorded mode, byte-identical build-log bundles across differing timing with differing bundle-mode runs, and a `sorry` warning still failing the stage. r[octet.prove_raw_log_retention.reproducibility]
- [x] [serial] Add artifact-verify tests accepting a build-log bundle and rejecting a manifest or stray raw log. r[octet.prove_raw_log_retention.verification]
- [x] [serial] Run the focused and repository-required checks and save the transcripts under `evidence/`, then commit. r[octet.prove_raw_log_retention.consumer_job]

## Verification Coverage

- `Scenario: Build-log run writes no bundle log artifacts` -> `evidence/prove-tests.txt`
- `Scenario: Capture consumers still decide the stage verdict` -> `evidence/prove-tests.txt`
- `Scenario: Valid build-log bundle is accepted` -> `evidence/artifact-tests.txt`
- `Scenario: Contradictory build-log bundle is rejected` -> `evidence/artifact-tests.txt`
- `Scenario: Timing text changes only the build log` -> `evidence/prove-tests.txt`
- `Scenario: Consumer job derivation selects build-log retention` -> `evidence/consumer-job-eval.txt`
