# Proposal: Add machine-readable assurance profiles

## Why

Octet can enforce individual source, proof, dependency, and evidence rails, but it cannot describe how a component selects those mechanisms without implying cumulative proof. The workspace now defines assurance planes, mechanisms, profiles, cases, and verification scope. Octet needs a typed configuration surface that preserves those claim boundaries in generated artifacts.

The OnixOS lifecycle proof change is the first concrete consumer. It already requires Octet, Kamacite, Verus, Kani, Valence, and Cairn evidence, but no single machine-readable profile links the mechanism owners, assumptions, trusted computing base, required artifacts, validation commands, and non-claims.

## What Changes

- Add closed Nickel contracts for one assurance profile and its linked mechanisms, assumptions, trusted-base elements, and evidence requirements.
- Add positive and negative fixtures for structural and cross-reference failures.
- Add an `onixos-lifecycle-proof-v1` pilot profile that maps the active OnixOS lifecycle proof chain across the five workspace assurance planes.
- Export deterministic JSON and an existing Nickel Export BLAKE3 manifest without adding a second identity implementation.
- Package the profile and identity manifest as a first-class Octet flake output for a future revision-pinned OnixOS consumer.
- Register the profile family in Octet policy metadata and document the operator boundary.

## Impact

- **Files**: `standards/policies/assurance-profiles/`, `standards/policies/registry.ncl`, `flake.nix`, policy documentation, Octet documentation, and this Cairn change.
- **Compatibility**: ordinary `cargo octet check`, strict Rust profiles, policy artifact schemas, and existing evidence rails remain unchanged.
- **Testing**: Nickel typechecking and positive/negative fixtures, generated JSON freshness, BLAKE3 manifest freshness, package layout checks, policy validation, focused Nix checks, and Cairn gates.

## Non-goals

- Do not claim the profile proves that required evidence exists or passes.
- Do not make Flux, `cargo-geiger`, persistent collections, or another verifier a default requirement.
- Do not make Valence own architecture policy or make Cairn redefine producer evidence semantics.
- Do not update an OnixOS flake pin before the Octet profile is published at an immutable revision.
