## Implementation

- [x] [serial] I1 Add pure complete-plan normalization and equivalence checks. r[octet.architecture.features.equivalence]
- [x] [serial] I2 Connect the second bounded Cargo plan and preserve selector rejection. Update the consumer contract. r[octet.architecture.features.rejection]

## Verification

- [x] [serial] V1 Record the focused baseline before core edits. r[octet.architecture.features.evidence]
- [x] [serial] V2 Add pure and actual Cargo positive and negative regression controls. r[octet.architecture.features.equivalence] r[octet.architecture.features.rejection]
- [x] [serial] V3 Run both ignored compiler controls and the full deny-all consumer comparison. Retain the commands and exact results. r[octet.architecture.features.evidence]
- [x] [serial] V4 Run focused tests, strict Clippy, format, the compiler-target Nix check, and lifecycle gates. Confirm no generated DTO contract changes. r[octet.architecture.features.evidence]
- [x] [serial] V5 Inspect and execute spec sync, then validate accepted requirements. r[octet.architecture.features.evidence]

Observed results and replay commands: [verification evidence](verification.md).

## Verification Coverage

| Scenario | Required evidence |
|---|---|
| Complete reviewed row | Pure equivalence tests, actual Cargo plan, actual compiler coverage |
| Omitted feature or dependency | Changed-feature, changed-target, optional-dependency, and dependency-only rejection controls |
| Ambiguous or mismatched input | Selector tests and malformed graph controls |
| Missing actual compiler unit | Ignored compiler controls with one removed shard |
| Executed positive and negative controls | Full deny-all consumer pass and under-declared row rejection |
| Evidence lacks actual compilation | Publication review rejects planner-only completion |

## Completion Postconditions

Archive only after every task and acceptance check passes. Publish the verified branch and integrate without force-pushes.
Preserve other active changes and worktrees. Retain raw evidence in the primary repository before the evidence guard and teardown.
Noble remains a separate change with separate acceptance requirements.
