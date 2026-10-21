# Tasks: Immutable Mantle producer handoff

## Phase 1: Producer contract

- [ ] [serial] I1 Bind the existing external-fixture profile, provider, commands, bounds, output contract and non-claims in the registered Nickel job and ordinary derivation. r[octet.mantle_producer_handoff.contract]
- [ ] [serial] I2 Export only an immutable committed producer graph and complete source objects; preserve original Nix bytes and reject unsupported, mixed, incomplete and dynamic-output graphs. r[octet.mantle_producer_handoff.acquisition]
- [ ] [serial] I3 Document immediate Mantle adoption, producer ownership, original verification versus cache reuse, and bounded non-claims. r[octet.mantle_producer_handoff.boundary]

## Phase 2: Actual acceptance

- [ ] [serial] V1 Run the typed job/registry checks and meaningful positive/negative exporter fixtures, and retain an actual immutable graph/source export with exact identities. r[octet.mantle_producer_handoff.acquisition]
- [ ] [serial] V2 Execute the actual ordinary owner job and preserve supported external proof and valid artifact reports without replacing them with process-success or export observations. r[octet.mantle_producer_handoff.contract] r[octet.mantle_producer_handoff.boundary]
- [ ] [serial] V3 Run owner-pinned native validation and proposal/design/tasks gates, commit verified implementation, then record separately admitted sync/archive, post-archive validation and publication. r[octet.mantle_producer_handoff.boundary]

## Verification Coverage

- Exact profile and original supported proof -> I1, V2.
- Unsupported/mixed graph, missing edge, dynamic output and immutable source drift -> I2, V1.
- Complete real graph and original source bytes -> I2, V1.
- Export/cache/process success does not acquire theorem or Mantle release authority -> I3, V1, V2, V3.
