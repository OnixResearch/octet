## Why

Current architecture checks use source text for port declarations, implementations, external types, capability calls, and path scopes.

Text matching can accept comments, miss aliases, miss macro expansion, misread generic types, and confuse inactive configuration with compiled code.

Octet needs compiler facts before an architecture gate can reject these bypasses with bounded evidence.

## What Changes

- Add an opt-in rustc-backed architecture fact collector for projects with an active architecture policy.
- Define a Rust-owned, versioned Architecture IR with a generated Nickel contract.
- Record selected targets, Cargo packages, features, roles, items, types, public signatures, traits, implementations, constructors, calls, effects, macro origins, configuration state, and unresolved facts.
- Use canonical BLAKE3 identities for policy, source, target, toolchain, fact records, and complete Architecture IR artifacts.
- Add a pure normalizer that orders, deduplicates, validates, and classifies supplied compiler facts.
- Add a coverage manifest that compares expected production targets and source scopes with observed compiler units.
- Preserve source-text checks as lower-tier diagnostics until their consumers migrate.

## Impact

- **Specs**: new `compiler-architecture-ir` capability
- **Files**: rustc collector, Architecture IR DTOs, generated Nickel contract, Cargo runner shell, normalizer core, artifacts, fixtures, and documentation
- **Testing**: aliases, comments, macros, generic signatures, trait implementations, indirect constructors, inactive `cfg`, dynamic dispatch, incomplete target coverage, malformed facts, and deterministic replay

## Dependencies

This change depends on `opt-in-project-architecture-policy`. The collector uses its explicit policy, roles, source scopes, targets, and mode.

## Non-goals

- Do not evaluate the complete project architecture gate in this change.
- Do not claim stable rustc internal identifiers across toolchains.
- Do not claim complete MIR call graphs, runtime reachability, dependency behavior, or semantic correctness.
- Do not run the collector for projects without an architecture policy.

## Completion Evidence

The change is complete when positive and adversarial fixtures emit deterministic IR, and incomplete extraction produces an explicit coverage failure.