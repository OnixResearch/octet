# Proposal: Enforce nominal domain types

## Why

Octet documents named Rust types as the preferred boundary for identities, units, capabilities, secrets, and refined values. The current policy remains advisory.

`ambiguous_params` checks only a narrow set of free-function parameters. `numeric_units` checks local variable names. Neither check rejects primitive aliases or raw domain values in core APIs.

A blanket name-based gate creates noise around wire DTOs, FFI records, generated code, and plain configuration snapshots. Octet needs an explicit policy that can distinguish core domain values from compatibility representations.

## What Changes

- Add a Nickel-authored nominal-type policy registry with exact domain names, primitive representations, core scopes, wire scopes, and constructor requirements.
- Add a `primitive_domain_alias` lint for aliases such as `type UnitId = String` in declared domain scope.
- Add a `raw_domain_value` lint for declared semantic fields, parameters, results, enum payloads, and map keys that still use raw representations.
- Add a `newtype_invariant_bypass` lint for public refined-value fields and unchecked primitive conversions.
- Extend `ambiguous_params` to associated functions, methods, `String`, path types, and non-consecutive repeated primitive-like types.
- Extend `numeric_units` to parameters and fields while accepting unit-bearing newtypes.
- Add positive, negative, compatibility, and compile-fail UI fixtures.
- Stage enforcement through inventory, warning, and scoped strict-profile denial.

## Impact

- **Policy**: New Nickel contracts and generated checker input.
- **Rust**: New Dylint passes, pure classification helpers, diagnostics, registration, metadata, and configuration loading.
- **Profiles**: Reviewed additions to `bounded-systems` and `deterministic-core` after signal evidence passes.
- **Consumers**: Pilot inventories for Valence and Mantle without live sibling dependencies.
- **Documentation**: Lint reference, strict-profile guidance, downstream rollout, and exception rules.

## Non-goals

- Do not require a wrapper for every primitive or local variable.
- Do not reject raw values inside declared wire, FFI, generated, vendored, or protocol compatibility scopes.
- Do not infer authority, trust, secrecy, or semantic correctness from a type name.
- Do not promote heuristic suffix matches directly to errors.
- Do not change existing consumer wire formats through this lint package.
