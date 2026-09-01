## Why

Octet effect closure uses conservative source-token analysis. Macros, async code, trait dispatch, conditional compilation, and unresolved calls remain unknown.

MIRAI provides path-sensitive MIR abstract interpretation and explicit annotations. A bounded pilot can measure whether it adds useful evidence without replacing Charon or expanding default dependencies.

## What Changes

- Pin one MIRAI revision, compatible rustc toolchain, target, and Nix closure for a selected pilot corpus.
- Define exact supported pilot properties, inputs, outputs, budgets, and failure classes.
- Compare MIRAI observations with Charon semantic projections and source-token effect evidence without merging authority.
- Emit an `adopt`, `defer`, `reject`, or exact `blocked` decision receipt.

## Impact

- **Specs**: new `mirai-semantic-pilot` capability
- **Files**: pilot policy, Nix provider package, selected fixtures, normalization core, comparison artifacts, receipts, evidence, and documentation
- **Testing**: provider compatibility, positive and negative properties, unsupported constructs, timeout, malformed output, comparison disagreements, and default-path independence

## Dependencies

The pilot uses the accepted Charon semantic-analysis and effect-closure artifacts as comparison parents. It does not change the active Aeneas consumer-contract work.

## Non-goals

- Do not make MIRAI a starter or strict-profile dependency.
- Do not replace Charon, Verus, Kani, Aeneas, or source-token closure.
- Do not claim MIRAI soundness, complete Rust support, or whole-program correctness.
- Do not create a production evidence rail unless the pilot decision is `adopt`.

## Completion Evidence

The change is complete when the selected corpus produces reproducible comparison evidence and a deterministic adoption decision or exact blocked result.
