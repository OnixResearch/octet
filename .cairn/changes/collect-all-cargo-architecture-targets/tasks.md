## Implementation and verification

- [x] [serial] Preserve the observed baseline and Cargo graph ambiguity. [r[octet.architecture.targets.evidence]]
- [x] [serial] Implement exact pure unit observation matching and context v2. [r[octet.architecture.targets.binding]]
- [x] [serial] Replace library-only enumeration with bounded Cargo target planning. [r[octet.architecture.targets.coverage]]
- [x] [serial] Connect rustc session observations and reject unexpected selected units. [r[octet.architecture.targets.binding]]
- [x] [serial] Add positive and negative planner, collector, and coverage controls. [r[octet.architecture.targets.coverage]]
- [x] [serial] Run actual compiling all-target fixtures and inspect compiler effects. [r[octet.architecture.targets.evidence]]
- [x] [serial] Run tests, strict Clippy, generated-contract checks, and lifecycle gates. [r[octet.architecture.targets.evidence]]
- [x] [serial] Sync the accepted compiler-target specification. [r[octet.architecture.targets.evidence]]

## Mandatory completion postconditions

Archive locally after sync. Publish the verified branch. Preserve the unrelated unpublished plan on primary main. Remote-main integration needs reconciliation of that plan, so retain the worktree until integration and guarded teardown are complete.
