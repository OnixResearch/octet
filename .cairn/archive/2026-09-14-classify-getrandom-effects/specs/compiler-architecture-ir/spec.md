# Compiler randomness effect delta

## ADDED Requirements

### Requirement: Compiler-resolved getrandom effects
r[octet.compiler_architecture_ir.randomness_effects]

The collector MUST classify compiler-resolved `getrandom::fill`, `getrandom::fill_uninit`, `getrandom::u32`, and `getrandom::u64` calls as randomness effects.
Classification MUST use the compiler's defining crate and root-function shape, not the consumer alias in a diagnostic path.
Each effect MUST preserve the call's recorded definition path, caller and source span, stable fact identity, and every observing compilation unit.
Library-test configurations MUST retain production obligations.
Local same-named helpers, prefix near matches, and unrelated getrandom definitions MUST NOT acquire this classification solely from name resemblance.
The repair MUST NOT suppress calls, change source ownership, relax coverage, or fabricate execution evidence.

#### Scenario: Real registry randomness calls

- GIVEN a configured library that calls the four published getrandom entry points through an import alias
- WHEN the compiler collector observes both production library configurations
- THEN all four calls and matching randomness effects retain their caller, span, and unit memberships
- THEN the gate retains each production observer's forbidden randomness findings

#### Scenario: Similar names are not randomness primitives

- GIVEN local same-named helpers, unrelated getrandom definitions, or prefix near matches
- WHEN the classifier evaluates their resolved definitions
- THEN these definitions do not become randomness effects solely through a prefix match
