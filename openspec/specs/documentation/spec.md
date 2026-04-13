# Documentation Skill Coverage Specification

## Purpose

Documentation requirements that keep the Tiger Style agent skill aligned with
its reference document while staying skimmable.
## Requirements
### Requirement: Tiger Style Skill Coverage Crosswalk

The project MUST maintain an explicit crosswalk from the Tiger Style
reference document to the agent skill.

The crosswalk MUST identify, for each major normative rule or grouped
rule set in the reference, where that guidance appears in the skill.

The crosswalk MUST call out any intentionally reference-only material
as examples, rationale, or extended background.

#### Scenario: Reviewing a future skill edit

- GIVEN the skill has been edited
- WHEN a maintainer verifies coverage
- THEN the crosswalk shows whether any reference guidance was dropped

### Requirement: Skill Completeness Without Verbatim Duplication

The Tiger Style skill MUST remain skimmable while still covering all
major normative guidance from the reference.

The skill MAY group related rules into concise checklists.

The skill MUST provide explicit pointers to the reference for extended
examples, rationale, and long-form walkthroughs.

The skill MUST NOT rely on the reference to carry normative rules that
are absent from the skill.

#### Scenario: Agent uses skill only

- GIVEN an agent reads the skill but not the full reference
- WHEN it applies Tiger Style during coding or review
- THEN it still sees all major normative rules needed for correct application

### Requirement: Downstream Consumer Rollout Plan

The project MUST maintain a documented rollout plan for integrating
`tigerstyle` into a real consumer workspace before claiming downstream
readiness.

The rollout plan MUST:

- name the target workspace
- identify the initial lint subset enabled for that workspace
- explain how the workspace wires tigerstyle into flake checks or equivalent
  CI
- record which lint families are deferred because of known noise or missing
  project-specific tuning
- define promotion criteria for moving a deferred lint family into a later
  rollout phase

#### Scenario: Preparing the first aspen rollout

- GIVEN aspen is the first downstream consumer
- WHEN a maintainer follows the rollout plan
- THEN aspen can run a scoped tigerstyle check without blindly enabling the
  full lint catalog

### Requirement: Consumer Baseline And Configuration Evidence

The project MUST capture baseline evidence and required configuration
adjustments for each downstream workspace onboarded through the rollout plan.

The integration notes MUST record:

- baseline violation counts or representative failures from the initial run
- required `dylint.toml` thresholds or allowlists
- rationale for each committed threshold, allowlist, or explicit exception used
  by the rollout
- validation steps showing the configured check runs end to end in the
  consumer workspace

#### Scenario: Reviewing a tuned aspen configuration

- GIVEN aspen needs workspace-specific thresholds or lint allowlists
- WHEN a maintainer reviews the integration notes
- THEN the reviewer can see why the config exists and how the check was
  validated

