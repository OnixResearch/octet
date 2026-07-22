# Restore effect-closure traceability

## Why

The accepted `cairn/specs/effect-closure-admission/spec.md` remains current and its implementation, tests, dogfood evidence, and operator documentation remain in the repository. The Cairn traceability profiles do not select that accepted spec or its exact implementation/docs roots, so the visible `octet.effect_closure.cli.check_failure` verification marker is reported as dangling while the rest of the accepted capability is invisible to coverage.

Deleting that marker would hide an implemented fail-closed path. Registering only one scenario is impossible without fragmenting the accepted source and would ignore the other 29 accepted effect-closure IDs. The repair must restore the complete accepted capability and fail if any requirement remains missing or dangling.

## What Changes

- Add the accepted effect-closure specification to both typed Nickel requirement-source lists.
- Add only the exact effect-closure core, shell, CLI parser/test, and operator doc roots needed by the accepted capability.
- Add the three missing parent/boundary markers whose child scenarios already have maintained evidence.
- Add a hermetic Nix gate that runs Cairn traceability against the real repository snapshot and against a negative copy with both direct-check failure markers removed.
- Preserve all accepted requirement text, runtime behavior, evidence tiers, and non-claim boundaries.

## Impact

The generated Cairn policy hash and flake check inventory change. No runner behavior, closure algorithm, receipt schema, or accepted requirement semantics change. Completion means both `missing` and `dangling` are empty and the negative fixture proves that removing semantic evidence fails closed; it does not make traceability coverage a correctness or release-eligibility proof.
