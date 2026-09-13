# Authority-scoped receipt identities

## Boundary and ownership

Octet owns the architecture receipt producer and verifier.
Both are pure functions over caller-supplied policy, compiler facts, and receipt values.
The runner owns collection, file I/O, and exit status. No runner behavior or DTO shape changes here.

`evaluate_project_architecture` selects authorization type and fact identities from protected protocol definitions.
An unprotected-only policy therefore produces empty authorization identity arrays.
`verify_project_architecture_receipt` currently rejects those arrays whenever any outbound effect exists.

## Decision

Keep every existing receipt check. Narrow only the authorization-identity precondition to the presence of at least one `Protected` classification.
Use the authority discriminator, not the optional protocol payload, the first effect, or the availability of identities.
Protected policies with a missing protocol or missing compiler evidence must not gain acceptance from this correction.
Mixed policies retain the protected identity requirement regardless of declaration order.

Keep all required non-claims for unprotected effects.
Do not add dummy witnesses, rewrite the caller policy, weaken compiler coverage, or switch to advisory mode.
Keep receipt schema `octet-project-architecture-receipt/v1` and its generated contract unchanged.
Independent artifact verification still binds the expected policy and other parent inputs.

## Verification

The tests construct ordinary policy and compiler values using the existing gate fixtures.
No external effect runs during these unit tests.

| Case | Expected observation |
|---|---|
| Existing gate baseline | Existing project-architecture tests pass before edits |
| Unprotected outbound effect | Clean receipt, empty authorization arrays, successful round trip and self-verification |
| Protected and mixed effects | Valid receipts pass; independently clearing either identity array still fails after resealing |
| Authority promotion | Changing unprotected to protected without identities fails after resealing |
| Identity tampering | Changed declaration without resealing fails identity verification |
| Missing non-claim | Each removed authorization-related non-claim fails after resealing |
| Unclassified effect | A declaration for a different family does not hide an unclassified actual effect |

The new unprotected positive test must fail against the original verifier.
Focused tests, the standards package tests, formatting, strict Clippy, and a Nix runner build form the provider acceptance scope.
Relevant Nix policy and traceability checks run before publication.
Full Octet external-proof rails and unrelated active changes are outside this repair; no whole-repository release claim is made.

Noble's original failed Nix and pre-commit runs remain in `.pi/m1-quality/`.
Provider commands and results remain in `.pi/m1-octet-repair/`, outside both disposable worktrees.
After publication, Noble must repin and rerun its unchanged policy and full deny-all scope before its blocker can close.
