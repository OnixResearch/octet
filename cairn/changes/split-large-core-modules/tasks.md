## Contract

- [ ] [serial] Specify pure-core and imperative-shell boundaries for runner modules. r[octet.runner_core_architecture.boundaries]
- [ ] [serial] Specify extraction expectations for current large hotspots. r[octet.runner_core_architecture.hotspot_extraction]
- [ ] [serial] Specify reusable test-support boundaries for large integration fixtures. r[octet.runner_core_architecture.test_support]

## Implementation

- [ ] [serial] Extract pure object/supplement/proof parsing and validation cores from large runner files. r[octet.runner_core_architecture.boundaries]
- [ ] [serial] Reduce CLI and filesystem modules to thin orchestration shells around those cores. r[octet.runner_core_architecture.hotspot_extraction]
- [ ] [serial] Move repeated integration fixture builders into named test-support modules. r[octet.runner_core_architecture.test_support]

## Verification

- [ ] [serial] Add positive core tests for valid object, proof, CLI-parse, and supplement inputs. r[octet.runner_core_architecture.boundaries]
- [ ] [serial] Add negative core tests for malformed inputs, missing fields, unsupported modes, and inconsistent receipts. r[octet.runner_core_architecture.hotspot_extraction]
- [ ] [serial] Run targeted cargo-octet tests, consumer integration tests touched by helper moves, and Cairn validation/gates. r[octet.runner_core_architecture.test_support]
