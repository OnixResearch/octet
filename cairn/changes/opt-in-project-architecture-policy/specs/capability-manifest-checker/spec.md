# Capability Manifest Checker Specification Delta

## MODIFIED Requirements

### Requirement: Capability manifest schema

r[octet.capability_manifest_checker.manifest] An activated project architecture policy MAY declare typed capability classifications across domain, application, adapter, composition-root, test, generated, and remote-execution scopes.

#### Scenario: Valid project manifest exports deterministic runtime policy

r[octet.capability_manifest_checker.manifest.valid]

- GIVEN an active Nickel project policy declares supported scope classes, capability families, application-owned ports, shell interpreters, deterministic handlers, and enforcement mode
- WHEN policy export and artifact freshness validation run
- THEN the manifest MUST export deterministic runtime JSON with a BLAKE3 identity and registry coverage
- AND the export MUST preserve the project activation identity

#### Scenario: Invalid active manifest fails closed

r[octet.capability_manifest_checker.manifest.invalid]

- GIVEN an active manifest omits required fields, names an unknown capability family, or declares an invalid strict scope
- WHEN policy validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field

#### Scenario: Project has no architecture policy

- GIVEN a workspace declares no architecture-policy pointer
- WHEN capability policy resolution runs
- THEN project capability evaluation MUST remain inactive
- AND Octet MUST NOT use global fallback source scopes

### Requirement: Scope resolution

r[octet.capability_manifest_checker.scope_resolution] Octet MUST resolve declared project paths and compiler units to project capability roles before applying capability obligations.

#### Scenario: Scope resolution is deterministic

r[octet.capability_manifest_checker.scope_resolution.deterministic]

- GIVEN the same active policy, repo-relative inventory, targets, features, and compiler units
- WHEN scope resolution runs twice
- THEN each selected item MUST resolve to the same role and capability classification
- AND ambiguous strict matches MUST fail closed

### Requirement: Effect observation matching

r[octet.capability_manifest_checker.effect_matching] Octet MUST compare observed direct effects, explicit domain facts, application-owned ports, effect plans, and shell interpreters against the active project policy.

#### Scenario: Undeclared direct effect is flagged

r[octet.capability_manifest_checker.effect_matching.direct_effect]

- GIVEN compiler facts place a direct platform effect in a strict domain-core scope
- WHEN capability evaluation runs
- THEN the checker MUST emit a deterministic diagnostic naming the effect family, scope, and required shell boundary

#### Scenario: Explicit fact and effect plan are accepted

r[octet.capability_manifest_checker.effect_matching.injected]

- GIVEN domain-core code receives explicit validated facts and returns a declared effect plan without invoking an external capability port
- WHEN capability evaluation runs
- THEN the domain-core shape MUST be accepted
- AND the application shell MAY interpret the plan through an application-owned port

#### Scenario: Domain core calls an injected infrastructure capability

- GIVEN domain-core code invokes a clock, repository, executor, network, filesystem, or other external capability trait
- WHEN strict capability evaluation runs
- THEN the checker MUST flag the call even when the concrete implementation is injected
- AND remediation MUST move the port invocation to the application shell