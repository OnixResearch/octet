# Compiler-owned caller identity

## Decision

Use rustc's definition kind before a lazy caller lookup creates an item.
Use the same item-kind mapping as existing local identity calculation.
Only function kinds can request a function signature. Inline and anonymous constants are constant items.
Existing closure identity handling remains separate and unchanged.
Expression body owners have private visibility. They must not request rustc's named-item visibility query.
The first repair removed the invalid signature query but exposed this second invalid query. Both failures remain in the operator evidence.

The rustc adapter owns this translation. Pure IR normalization still owns identity and reference validation.
No new port, dependency, runtime DTO, or Nickel contract is needed.
The existing policy registry records the additional compiler fixtures.

## Rejected alternatives

Skipping constant bodies loses calls and effects. Using an enclosing function invents a different owner.
Catching the compiler panic cannot reconstruct trustworthy observations. A missing signature is not permission to omit an item.

## Validation

Run the collector unit baseline before implementation.
Add actual Cargo compiler regressions and retain the old-library failure before the repair.
Cover inline, anonymous, named, associated, and macro-generated constants, static initializers, closures, and normal functions.
Require matching call references and source-bound item kinds in both library production configurations.
Require constant owners to have no function signature. Preserve ordinary function signatures and forbidden filesystem observations.
Keep full unit binding, membership, coverage, and negative missing-shard checks.

Run relevant Rust tests, formatting, strict Clippy, compiler-target Nix checks, and a fresh provider build.
Use a rebuilt collector library for compiler and Noble fixture probes. Do not infer new collector behavior from an old library.
Keep all evidence under the primary Noble repository's `.pi/m1-inline-owner/` directory.
Publish the provider before Noble changes its immutable pins. Noble task 2.2 remains open until its own required controls pass.

## Risks and limits

Lazy and eager discovery must agree on identities. Anonymous constants must not acquire transient HIR or definition indices as identifiers.
Macro bodies must retain observations rather than gain exemptions. Unknowns remain explicit and cannot imply purity.
The JSON runner's internal-error severity failure is a secondary diagnostic issue. It cannot be promoted to a clean result.
The selected compiler regressions are not an exhaustive Rust body inventory.
