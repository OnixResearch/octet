## Phase 1: Integrated orchestration

- [ ] [serial] Add the optional architecture phase to `cargo octet check` after project-policy activation r[octet.project_architecture_gate.orchestration]
- [ ] [serial] Keep lint, architecture, and effect-closure findings, phase status, evidence tier, and non-claims separate r[octet.project_architecture_gate.orchestration]
- [ ] [serial] Treat policy, Cargo metadata, Architecture IR, coverage, target, and feature input failures as integration failures r[octet.project_architecture_gate.input_integrity]
- [ ] [serial] Preserve byte-compatible no-policy behavior and artifacts r[octet.project_architecture_gate.compatibility]

## Phase 2: Architecture evaluation core

- [ ] [serial] Implement the pure role-aware dependency graph with a visible composition-root role r[octet.project_architecture_gate.role_graph]
- [ ] [serial] Evaluate complete role coverage, forbidden edges, domain-core purity, application-owned ports, adapter implementations, and composition-root construction r[octet.project_architecture_gate.fcis]
- [ ] [serial] Evaluate external-type leakage, capability classification, optional `no_std`, and required unknown compiler facts r[octet.project_architecture_gate.fcis]
- [ ] [serial] Implement `inventory`, `advisory`, `gate`, and `admission` outcome rules r[octet.project_architecture_gate.modes]
- [ ] [serial] Compose admission mode with the existing effect-closure receipt without merging claims r[octet.project_architecture_gate.admission]
- [ ] [serial] Apply exact BLAKE3-bound waivers and retain `waived` findings in output r[octet.project_architecture_gate.waivers]

## Phase 3: Evidence and consumer surfaces

- [ ] [serial] Add the Rust-owned architecture receipt and generated Nickel contract r[octet.project_architecture_gate.evidence]
- [ ] [serial] Bind policy, command, Cargo graph, Architecture IR, coverage, roles, findings, unknowns, waivers, phase status, target, and toolchain r[octet.project_architecture_gate.evidence]
- [ ] [serial] Extend artifact verification for architecture receipt identity, links, status, and tamper detection r[octet.project_architecture_gate.evidence]
- [ ] [parallel] Update CLI help, consumer docs, templates, status, summary, JSON, SARIF, and provenance surfaces r[octet.project_architecture_gate.compatibility]

## Phase 4: Verification

- [ ] [parallel] Add positive fixtures for all roles, valid ports, composition roots, pure transitions, waivers, modes, and admission r[octet.project_architecture_gate.verification]
- [ ] [parallel] Add adversarial fixtures for narrowed scope, comments, aliases, macro effects, inactive adapters, hidden helpers, unresolved dispatch, and missing metadata r[octet.project_architecture_gate.verification]
- [ ] [parallel] Add negative fixtures for omitted targets, missing shards, shell construction, stale waivers, phase masking, and artifact tampering r[octet.project_architecture_gate.verification]
- [ ] [serial] Run focused runner and standards tests, receipt verification, docs checks, Cairn validation, gates, traceability, and full compatible checks r[octet.project_architecture_gate.verification]