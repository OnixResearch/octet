## Phase 1: Input and lifecycle migration

- [x] [serial] Generate the targeted Cairn lock update and verify the exact admitted revision. a[update-cairn-input.lock]
- [x] [serial] Preview and execute the hash-bound lifecycle migration, then verify old/new byte parity. a[update-cairn-input.migration]
- [x] [serial] Retire the legacy tree and update maintained Octet paths to `.cairn/`. a[update-cairn-input.paths]

## Phase 2: Validation and closeout

- [x] [serial] Run traceability, policy, focused Octet, and full flake checks against the baseline. a[update-cairn-input.validation]
- [x] [serial] Save bounded evidence, complete review, and archive this no-spec change. a[update-cairn-input.validation]
