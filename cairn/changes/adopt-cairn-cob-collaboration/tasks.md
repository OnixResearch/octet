## Phase 1: Lifecycle and signed import

- [x] [serial] Define and gate the bounded Octet adoption contract before COB mutation. r[octet.cob_collaboration.pilot]
- [x] [serial] Seed an explicit bare Git store and capture preview plus executed signed import with an ephemeral mutation key. r[octet.cob_collaboration.pilot.import]

## Phase 2: Downstream implementation and update

- [x] [serial] Add the operator guide, typed Nickel manifest source, and pure-core positive/negative evidence validator. r[octet.cob_collaboration.evidence]
- [x] [serial] Preview and execute a non-noop signed update after the active lifecycle artifacts change. r[octet.cob_collaboration.pilot.update]

## Phase 3: Explicit transport and second checkout

- [x] [serial] Move exact branch and namespace refs with local Git only, verify source/target identity and no store remotes, then capture second-store status and conflict inspection. r[octet.cob_collaboration.transport]
- [x] [serial] Preview and execute checkout into a second Octet root and compare materialized active-change bytes. r[octet.cob_collaboration.checkout]

## Phase 4: Validation and archive

- [x] [serial] Run manifest freshness, focused positive/negative tests, Cairn gates, Octet policy/check rails, and bounded Nix validation while preserving baseline limitations. r[octet.cob_collaboration.validation]
- [ ] [serial] Sync the accepted requirement, archive the completed change with a deterministic date, and record exact non-claims. r[octet.cob_collaboration.boundary]
