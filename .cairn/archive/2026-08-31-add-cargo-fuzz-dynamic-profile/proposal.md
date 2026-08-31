## Why

Octet has typed dynamic-evidence profiles for Miri and Loom. It has no first-class profile for coverage-guided fuzzing or reproducible crash intake.

A bounded `cargo-fuzz` profile fills the gap between sampled tests and scoped model checks. It also preserves corpus and reproducer identity for review.

## What Changes

- Add an opt-in `cargo-fuzz-libfuzzer` profile to the existing dynamic-evidence registry.
- Bind the selected target, engine, toolchain, command, budgets, corpus, dictionary, crash artifacts, and minimized reproducer into a receipt.
- Add positive and negative consumer fixtures for Miri, Loom, and cargo-fuzz profile execution.
- Keep every dynamic profile outside starter `cargo octet check`.

## Impact

- **Specs**: `dynamic-evidence-rails`
- **Files**: dynamic profile Nickel, generated profile data, evidence-rail DTOs, runner shell, fixtures, artifact validation, and consumer documentation
- **Testing**: profile export, clean fuzz run, crash discovery, minimized replay, malformed input, stale identity, command rejection, and default-path independence

## Non-goals

- Do not make fuzzing a default Octet requirement.
- Do not claim exhaustive input coverage, panic freedom, memory safety, or formal proof.
- Do not grant arbitrary command authority through profile configuration.
- Do not replace Kani, Hegel, Miri, Loom, or ordinary tests.

## Completion Evidence

The change is complete when bounded clean and crash fixtures produce deterministic receipts, malformed cases fail closed, and starter checks remain unchanged.
