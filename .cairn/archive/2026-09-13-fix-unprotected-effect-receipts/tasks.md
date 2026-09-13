# Unprotected-effect receipt repair

All evidence is retained in `/home/brittonr/git/noble/.pi/m1-octet-repair/`.
This checklist covers the provider correction, not Noble M1 acceptance.

- [x] [serial] V1 Run the existing architecture-gate test baseline and planning gates. r[octet.project_architecture_gate.authority_scoped_receipt_ids]
- [x] [serial] I1 Add unprotected, protected, mixed, tampering, missing-non-claim, and unclassified-effect regression cases. r[octet.project_architecture_gate.authority_scoped_receipt_ids]
- [x] [serial] V2 Run the new regression against the original verifier and retain its expected failure. r[octet.project_architecture_gate.authority_scoped_receipt_ids]
- [x] [serial] I2 Correct only the authorization-identity precondition and document the contract. r[octet.project_architecture_gate.authority_scoped_receipt_ids]
- [x] [serial] V3 Run focused and standards-package tests, formatting, strict Clippy, relevant Nix checks, and the runner build. r[octet.project_architecture_gate.authority_scoped_receipt_ids]
- [x] [serial] H1 Retain provider acceptance evidence: command=`nix develop -c cargo test -p octet-standards --lib project_architecture_gate::tests --locked --offline -- --nocapture`; input=reviewed verifier and regression fixtures with the Nix vendor setup in evidence.md; asserts=unprotected receipt acceptance with unchanged protected and claim-boundary rejection; evidence=/home/brittonr/git/noble/.pi/m1-octet-repair/. r[octet.project_architecture_gate.authority_scoped_receipt_ids]

The [evidence record](evidence.md) scopes acceptance to this repair.
The supplementary pedantic run has 21 pre-existing findings in unchanged Architecture IR code. It is not a passing full-repository gate.

## Verification Coverage

| Scenario | Test surface |
|---|---|
| Unprotected receipt has no protected identities | Unprotected positive, round-trip, and deterministic receipt tests |
| Protected requirement survives a mixed effect list | Resealed missing-fact, missing-type, mixed-order, and authority-promotion controls |
| Unprotected classification does not bypass other checks | Identity tampering, resealed missing non-claims, and unclassified-effect controls |

## After implementation acceptance

Inspect sync and archive dry runs before execution. Validate the archived change before committing and publishing.
Keep unrelated active provider changes untouched. Preserve both worktrees until their evidence is retained.
