# Correct unprotected-effect receipt validation

## Why

Noble's deny-all architecture run reaches receipt self-verification and fails at Octet revision `0b1ca30a7e86bb38369bdf2b8a851772a3b9f213`.
The policy explicitly declares unprotected console output without a protected authorization protocol.
The producer selects authorization identities from protected protocol types only.
The verifier incorrectly requires those identities for every nonempty outbound-effect list.

## What Changes

- Require authorization fact and type identities only when at least one outbound effect is protected.
- Preserve identity, non-claim, schema, evidence-tier, and protected-effect checks.
- Cover unprotected, protected, mixed, tampered, and unclassified cases with deterministic tests.
- Document the corrected contract without a new receipt schema or consumer suppression.

## Impact

The pure verifier in `crates/octet-standards/src/project_architecture_gate.rs` owns this correction.
No lint, policy declaration, compiler scope, or authority boundary is removed.
The existing `project-architecture-gate` capability gains one explicit receipt requirement.
The implementation remains a published provider contract; Noble must select the resulting immutable revision separately.

## Completion

Acceptance requires a failing regression before the fix, passing focused and package checks afterward, and a built published runner.
The source and exact command logs remain in Noble's persistent `.pi/m1-octet-repair/` directory.
Sync and archive follow the completed checklist. Normal branch and main publication follow verification.
This change does not complete Noble M1 or establish runtime authorization, execution success, or proof claims.
