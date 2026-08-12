## Why

FCIS separation can keep effects outside the domain core while still allowing the shell to execute protected effects before an authority decision.

An effect plan can also be mislabeled as an execution receipt. These shapes violate the Onix authority and evidence boundaries.

Configured projects need a type protocol that makes authorization and observation visible to the compiler architecture gate.

## What Changes

- Extend project architecture policy with protected effect families, authority-decision constructors, authorization witness types, protected executors, observed-outcome types, and receipt constructors.
- Require opaque authorization witnesses with private fields and no ambient construction path.
- Reject default, deserialization, broad conversion, public unchecked constructor, and unsafe construction paths for authorization witnesses.
- Require every protected executor operation to consume or borrow the declared witness type.
- Require authorization witnesses to originate from declared domain decision outputs within the checked compiler graph.
- Keep effect plans, execution attempts, observed outcomes, and execution receipts as distinct types and identities.
- Reject receipt construction from a plan or attempt without a declared observed outcome.
- Add positive and negative compiler fixtures for authorization, denial, forgery, stale policy, pre-decision execution, and planned-effect receipt promotion.

## Impact

- **Specs**: new `authorized-effect-protocol` capability
- **Files**: architecture policy contract, compiler facts, pure protocol evaluator, diagnostics, receipts, templates, docs, and fixtures
- **Testing**: approved execution, denied execution, missing witness, forged witness, permissive derives, broad conversions, unauthorized constructor, receipt-from-plan, missing observation, and valid observed receipt

## Dependencies

This change depends on `integrate-project-architecture-gate`. It uses the project policy, Architecture IR, and integrated receipt path.

## Non-goals

- Do not move product authority policy into Octet.
- Do not define product credentials, revocation, replay, trust, or actor discovery.
- Do not prove that a declared decision function implements correct business policy.
- Do not claim that an execution receipt proves external-system truth beyond the recorded observation.

## Completion Evidence

The change is complete when declared protected executors cannot pass the architecture gate without a valid witness path and observed receipt boundary.