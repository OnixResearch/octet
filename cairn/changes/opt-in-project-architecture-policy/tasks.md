## Phase 1: Activation and policy contract

- [ ] [serial] Add `[workspace.metadata.octet].architecture_policy` parsing with workspace-relative path validation and no-policy compatibility r[octet.project_architecture_policy.activation]
- [ ] [serial] Add the typed Nickel project policy contract, consumer template, deterministic export, and BLAKE3 freshness manifest r[octet.project_architecture_policy.schema]
- [ ] [serial] Add the role, production-scope, target, dependency-rule, port, capability-classification, waiver, and non-claim fields r[octet.project_architecture_policy.roles]
- [ ] [serial] Add deterministic `inventory`, `advisory`, `gate`, and `admission` mode validation r[octet.project_architecture_policy.modes]

## Phase 2: Existing architecture policy migration

- [ ] [serial] Remove consumer fallback activation from crate-boundary, port, and capability policy resolution r[octet.project_architecture_policy.activation]
- [ ] [serial] Add and select an explicit Octet self-policy that preserves intended first-party checks r[octet.project_architecture_policy.compatibility]
- [ ] [serial] Rename port ownership from core scope to application-owner scope with deterministic migration diagnostics r[octet.project_architecture_policy.roles]
- [ ] [serial] Keep `deterministic-core` strict profile behavior independent from project architecture activation r[octet.project_architecture_policy.compatibility]
- [ ] [serial] Implement exact architecture-waiver validation and stale-binding diagnostics r[octet.project_architecture_policy.waivers]

## Phase 3: Consumer surfaces

- [ ] [parallel] Add `templates/octet-architecture.ncl` with one role-complete example and no ambient sibling paths r[octet.project_architecture_policy.schema]
- [ ] [parallel] Document activation, modes, roles, application-owned ports, optional `no_std`, waivers, and no-policy non-claims r[octet.project_architecture_policy.compatibility]
- [ ] [parallel] Update generated artifact and config-receipt registries for the Nickel-authored policy family r[octet.project_architecture_policy.schema]

## Phase 4: Verification

- [ ] [parallel] Add positive fixtures for no-policy compatibility, every mode, every role, explicit self-policy, and a current waiver r[octet.project_architecture_policy.verification]
- [ ] [parallel] Add negative fixtures for escaping paths, malformed exports, unknown modes or roles, overlap, missing production coverage, and stale waivers r[octet.project_architecture_policy.verification]
- [ ] [serial] Prove unconfigured command, finding, and artifact behavior remains unchanged r[octet.project_architecture_policy.compatibility]
- [ ] [serial] Run focused policy tests, generated-artifact freshness, docs checks, Cairn validation, gates, and traceability coverage r[octet.project_architecture_policy.verification]