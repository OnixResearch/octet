# Proposal: Add proof robustness evidence

## Why

A passing proof receipt records one verifier outcome. It does not show whether the proof survives controlled solver-seed changes or resource variation.

Octet also lacks one typed view of declared assumptions, observed axiom use, and provider dependency coverage. Reviewers must combine logs, source scans, and trust-boundary prose by hand.

Dafny release `v4.11.0` at commit `fcb2042d6d043a2634f0854338c08feeaaaf4ae2` provides the design reference. Its complexity, audit, and proof-dependency mechanisms motivate this change. Octet will adapt the mechanisms to existing proof rails instead of adding Dafny to normal builds.

## What Changes

- Add a typed Nickel proof-robustness profile for selected existing proof rails.
- Run a declared solver-seed matrix through the existing bounded proof command shell.
- Classify outcome stability, resource stability, incomplete runs, and unsupported provider facts.
- Record declared assumptions, trust boundaries, observed axiom use, and dependency-coverage class.
- Emit a Rust-owned robustness receipt linked to every child proof receipt and exact source, verifier, solver, toolchain, command, and profile identity.
- Export robustness evidence through existing artifact and Valence handoff paths without promoting its evidence role.
- Add positive and negative fixtures for stable proofs, changed outcomes, resource drift, stale identities, missing assumptions, unsupported dependency data, and overclaims.

## Impact

- **Specs**: new `proof-robustness-evidence` capability.
- **Files**: Nickel profiles, generated runtime data, runner core models, proof command orchestration, receipt DTOs, artifact validation, fixtures, and documentation.
- **Compatibility**: existing Verus, Aeneas, Kani, solver, and starter paths remain unchanged unless the operator selects a robustness profile.
- **Licensing**: the design uses clean-room adaptation from MIT-licensed Dafny. No Coco/R source or generated parser material enters Octet.

## Dependencies

This change extends accepted Verus proof rails, proof artifact identity, assurance profiles, runner core boundaries, and Valence export behavior.

The first provider is Verus. A direct Dafny provider requires a later Cairn change with its own Nix, trust, license, and output-contract review.

## Non-goals

- Do not claim solver independence, proof soundness, proof completeness, semantic equivalence, or whole-program correctness.
- Do not treat stable resource use as proof validity.
- Do not infer complete proof dependencies from Verus axiom-usage output.
- Do not make Dafny, Verus robustness runs, or any solver mandatory for starter `cargo octet check`.
- Do not create a new evidence role or bypass existing proof-receipt validation.

## Completion Evidence

The change is complete when selected positive proofs produce deterministic robustness receipts and every declared negative fixture fails at its expected boundary.
