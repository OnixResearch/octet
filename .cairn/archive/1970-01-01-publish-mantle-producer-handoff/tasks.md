# Tasks: Immutable Mantle producer handoff

## Phase 1: Producer contract

- [x] [serial] I1 Bind the existing external-fixture profile, provider, commands, bounds, output contract and non-claims in the registered Nickel job and ordinary derivation. r[octet.mantle_producer_handoff.contract]
- [x] [serial] I2 Export only an immutable committed producer graph and complete source objects; preserve original Nix bytes and reject unsupported, mixed, incomplete and dynamic-output graphs. r[octet.mantle_producer_handoff.acquisition]
- [x] [serial] I3 Document immediate Mantle adoption, producer ownership, original verification versus cache reuse, and bounded non-claims. r[octet.mantle_producer_handoff.boundary]

## Phase 2: Actual acceptance

- [x] [serial] V1 Run the typed job/registry checks and meaningful positive/negative exporter fixtures, and retain an actual immutable graph/source export with exact identities. r[octet.mantle_producer_handoff.acquisition]
- [x] [serial] V2 Execute the actual ordinary owner job and preserve supported external proof and valid artifact reports without replacing them with process-success or export observations. r[octet.mantle_producer_handoff.contract] r[octet.mantle_producer_handoff.boundary]
- [x] [serial] V3 Run owner-pinned native validation and proposal/design/tasks gates, publish the verified implementation, and retain native sync/archive plans for separate terminal execution. r[octet.mantle_producer_handoff.boundary]

The implementation is published at `9237f931e8e65dd4254011839d5fd207f2ad85a5`.
Actual immutable owner-job execution, original artifact verification, tamper
rejection, and incompatible-wire rejection are retained in
`evidence/owner-acceptance-v10/`. Native validation and all three gates passed
in `native-readiness-resume30`; its archive preview correctly remained blocked
on these then-unchecked tasks.

Terminal completion additionally requires separately recorded native sync and
archive execution, post-archive validation, and publication of the archive.
The task checkboxes are not evidence that those later shell operations happened.

## Verification Coverage

- Exact profile and original supported proof -> I1, V2.
- Unsupported/mixed graph, missing edge, dynamic output and immutable source drift -> I2, V1.
- Complete real graph and original source bytes -> I2, V1.
- Export/cache/process success does not acquire theorem or Mantle release authority -> I3, V1, V2, V3.
