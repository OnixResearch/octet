# Law Tested Deterministic Cores Specification Delta

## ADDED Requirements

### Requirement: Hegel laws use a checked value-based catalog

r[octet.law_tested_deterministic_cores.hegel_catalog]

Octet MUST maintain a checked catalog for selected Hegel laws that binds each pure core, law ID, generator, positive property, negative property, budget, replay method, and fixture complement.

#### Scenario: Selected core enters the catalog

- GIVEN generated inputs materially improve coverage for a deterministic in-memory core
- WHEN maintainers add a Hegel law
- THEN the catalog MUST name the generated-input benefit, positive property, negative property, budget, replay method, and complementary fixture
- AND the property body MUST NOT require filesystem, process, network, clock, or environment effects

#### Scenario: Law duplicates concrete coverage

- GIVEN a proposed Hegel law adds no generated-input benefit beyond existing concrete fixtures
- WHEN catalog validation runs
- THEN validation MUST reject the entry or require a documented diagnostic-only exemption

#### Scenario: Counterexample becomes a regression case

- GIVEN a cataloged Hegel law finds a minimal failing input
- WHEN maintainers accept the fix
- THEN the failure MUST remain replayable through stable Hegel output or a checked-in positive or negative regression fixture

#### Scenario: User-visible behavior remains covered

- GIVEN a cataloged law covers a core that affects diagnostics, CLI output, persisted artifacts, or source-level findings
- WHEN law coverage is reviewed
- THEN matching positive and negative UI, golden, transcript, or artifact fixtures MUST remain in place
