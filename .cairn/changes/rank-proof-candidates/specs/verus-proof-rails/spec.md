# Verus Proof Rails Delta

## ADDED Requirements

### Requirement: Proof-candidate discovery
r[octet.proof_candidates.selection] `cargo octet prove --candidates` MUST list Tiger Style lint allowances and waivers in the checked source that have no validated `proof-ref`, grouped by lint, and MUST drop a site once a reference binds that exact source path and line and validates against the supplied proof-card deck.

#### Scenario: Allowance has no proof-ref
- GIVEN a fixture crate with `#[allow(tigerstyle::unchecked_narrowing)]` at `src/codec.rs:42`
- WHEN `cargo octet prove --candidates` scans the crate
- THEN the allowance is listed with its lint, path, line, and missing proof-ref status

#### Scenario: Validated proof-ref discharges the allowance
- GIVEN the same allowance and a validated `proof-ref.json` whose consumer context names `src/codec.rs:42`
- WHEN the candidate listing runs with that reference and the matching proof-card deck
- THEN the allowance is absent from the candidate list and the discharged count is one

#### Scenario: Stale or unbound reference does not discharge
- GIVEN a proof-ref with a stale source BLAKE3 or a consumer context for another site
- WHEN the candidate listing runs
- THEN the allowance remains listed with `invalid` or `unbound` status and its diagnostics

### Requirement: Deterministic candidate ranking
r[octet.proof_candidates.ranking] The candidate report MUST rank sites by Tiger Style lint severity and by trust-boundary membership read from the machine-readable prefix block in `docs/trust-boundary-evidence.md`, and MUST produce a stable, sorted JSON document for repeated runs over the same inputs.

#### Scenario: Severity and trust boundary order candidates
- GIVEN candidates with different default lint levels and one path under a declared trust-boundary prefix
- WHEN the report is built
- THEN higher severity and trust-boundary membership produce a higher rank score
- AND equal scores sort by lint, path, line, column, and attribute

#### Scenario: JSON is stable
- GIVEN the same source roots, trust-boundary document, proof cards, and references
- WHEN the JSON report is generated twice
- THEN both serializations are identical and candidate ranks are contiguous from one

### Requirement: Existing-rail mapping and unsupported candidates
r[octet.proof_candidates.rail_mapping] The candidate report MUST map only rails that already exist in the checked-in proof-rail manifest, MUST state the mapped rail's scoped property limit, and MUST report every unmapped lint explicitly instead of implying a proof rail exists.

#### Scenario: Lint has an existing rail
- GIVEN a candidate whose lint is in the checked-in mapping table
- WHEN the report is built
- THEN the candidate names the proof/property ID, policy ID, rail index, and scope note

#### Scenario: Lint has no existing rail
- GIVEN a candidate whose lint is absent from the mapping table
- WHEN the report is built
- THEN the candidate has no rail, is marked unsupported, and the lint appears in the unsupported summary

### Requirement: Negative-check proof-card record
r[octet.proof_candidates.negative_check] `cargo octet prove --record-negative-check` MUST compare a baseline proof-card deck with a mutant-run deck, MUST record the result in the named proof card under `negative_check` in `proof-cards.json`, and MUST report a rail that still passes against the mutant as `property_not_demonstrated`.

#### Scenario: Mutant still passes the rail
- GIVEN a baseline card and a mutant-run card for the same proof/property ID with status `passed`
- WHEN the negative check is recorded
- THEN the card records `property_not_demonstrated` and the command exits non-zero

#### Scenario: Mutant fails the rail
- GIVEN a mutant-run card for the same proof/property ID with status `failed`
- WHEN the negative check is recorded
- THEN the card records `property_demonstrated` scoped to this mutation and the command exits zero

#### Scenario: Mutant run is inconclusive
- GIVEN a mutant deck without a comparable card for the proof/property ID
- WHEN the negative check is recorded
- THEN the card records `inconclusive`, explains the missing card, and the command exits non-zero

### Requirement: Candidate walkthrough skill
r[octet.proof_candidates.walkthrough] The repository MUST provide a `docs/skills/prove-a-candidate/SKILL.md` walkthrough that scaffolds or selects a rail, runs `prove --explain` and `prove`, writes and validates the proof-ref, runs `artifact verify`, runs the mutant negative check, and states the claim boundary at each step.

#### Scenario: Skill reaches a validated proof-ref
- GIVEN one listed candidate and an existing rail
- WHEN the skill steps are followed
- THEN the transcript shows proof-ref validation and artifact verification
- AND the negative-check result is recorded before any property claim

#### Scenario: Negative check refuses success
- GIVEN a rail that still passes against the mutant
- WHEN the skill reports the result
- THEN it reports `property_not_demonstrated` and does not claim the suppressed property is proven

### Requirement: Candidate validation fixtures
r[octet.proof_candidates.validation] Every new candidate, ranking, mapping, and negative-check behavior MUST include paired positive and negative tests, including a fixture crate whose allowance drops off after a validated proof-ref exists.

#### Scenario: Positive and negative tests run
- GIVEN the proof-candidate unit tests and fixture crate
- WHEN the focused test filter runs
- THEN valid and invalid references, mapped and unsupported lints, and passed and failed mutant cards are all exercised
