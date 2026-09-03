## Why

Octet currently separates Dylint checks from standards architecture checks. A project can run `cargo octet check` and omit FCIS or Hexagonal checks.

The standards shell can also omit dependency analysis when Cargo metadata is unavailable. This omission can produce a false-clean architecture result.

Configured projects need one command that gathers complete facts and evaluates one scoped architecture contract.

## What Changes

- Extend `cargo octet check` to run the project architecture rail only when `[workspace.metadata.octet].architecture_policy` is present.
- Keep lint and architecture findings, statuses, evidence tiers, and non-claims separate inside one parent result.
- Evaluate production coverage, role assignment, dependency direction, core purity, application-owned ports, adapter implementations, composition-root construction, external-type leakage, and capability classification.
- Add the `composition-root` role to permit visible selection of concrete adapters without allowing shell-owned infrastructure selection.
- Fail configured `gate` and `admission` runs on missing Cargo metadata, missing compiler facts, unknown production roles, unresolved required facts, omitted targets, or forbidden edges.
- Compose `admission` mode with the existing effect-closure rail.
- Evaluate exact BLAKE3-bound architecture waivers without treating a waived result as clean.
- Emit and verify one architecture receipt bound to policy, Cargo graph, Architecture IR, diagnostics, waivers, coverage, toolchain, target, and command.

## Impact

- **Specs**: new `project-architecture-gate`; modify `effect-closure-admission`
- **Files**: cargo-octet orchestration, standards evaluator, role graph core, result models, receipts, artifact verifier, templates, docs, and fixtures
- **Testing**: no-policy compatibility, all enforcement modes, complete role graphs, forbidden edges, omitted scopes, metadata errors, compiler gaps, port ownership, adapter placement, waiver drift, receipt tampering, and effect-closure composition

## Dependencies

This change depends on `opt-in-project-architecture-policy` and `emit-compiler-architecture-ir`.

## Non-goals

- Do not treat a lint-clean result as an architecture-clean result.
- Do not claim complete dependency implementation behavior, runtime determinism, whole-program correctness, or release eligibility.
- Do not require the architecture rail for projects without an explicit policy pointer.

## Completion Evidence

The change is complete when one configured command cannot return architecture-clean after any required policy, graph, compiler, scope, or target input is absent.