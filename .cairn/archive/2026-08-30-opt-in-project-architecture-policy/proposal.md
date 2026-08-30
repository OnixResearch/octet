## Why

Octet has FCIS, port, capability, and dependency checks. These checks use Octet-specific fallback paths when a consumer declares no architecture policy.

A project must choose its architecture contract. Octet must not infer a project claim from directory names or an empty global mechanism.

Activation must be opt-in. After activation, malformed policy and incomplete declared scope must fail closed.

## What Changes

- Add a project-owned `octet-architecture.ncl` policy with a typed contract and deterministic runtime export.
- Add `[workspace.metadata.octet].architecture_policy` as the native Cargo pointer to that Nickel source.
- Define `inventory`, `advisory`, `gate`, and `admission` enforcement modes.
- Define `domain-core`, `application-shell`, `inbound-adapter`, `outbound-adapter`, `composition-root`, `test`, and `generated` roles.
- Require an explicit production scope, role assignments, dependency rules, port ownership, capability classifications, and governed waivers.
- Disable project architecture checks and architecture claims when the project has no architecture-policy pointer.
- Replace Octet-layout fallbacks with an explicit Octet self-policy that preserves current first-party behavior.
- Keep strict Rust profiles independent. Selecting `deterministic-core` alone does not activate a project architecture claim.

## Impact

- **Specs**: add `project-architecture-policy`; modify `crate-boundary-config`, `hexagonal-ports`, and `capability-manifest-checker`
- **Files**: Nickel contracts, a consumer template, generated runtime policy, Cargo metadata parsing, standards policy resolution, docs, fixtures, and freshness artifacts
- **Testing**: absent policy, valid policy, malformed policy, unknown role, overlapping scope, missing production coverage, waiver drift, explicit Octet self-policy, and unchanged unconfigured projects

## Dependencies

This is the first change in the project architecture sequence. Later changes consume this policy through compiler facts and an integrated gate.

## Non-goals

- Do not add compiler architecture fact extraction in this change.
- Do not merge `cargo octet check` with the standards architecture checker in this change.
- Do not claim a complete FCIS, Hexagonal, effect, authority, or runtime proof.
- Do not force architecture policy on existing consumers.

## Completion Evidence

The change is complete when unconfigured projects retain byte-compatible behavior and configured projects receive deterministic policy activation, scope, role, and waiver results.