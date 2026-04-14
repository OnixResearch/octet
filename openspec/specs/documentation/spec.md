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

### Requirement: Reusable Consumer Integration Kit

After the first real downstream rollout, the project MUST generalize the
consumer-specific lessons into a reusable onboarding kit for later consumers.

The integration kit MUST include:

- named starter rollout profiles with rationale for why they are suitable as
  first adopters
- a reusable check-path recipe for consumers whose toolchain or environment
  needs Dylint wrappers or extra setup
- a sample `dylint.toml` layout with commented threshold, allowlist, and
  rationale sections
- a checklist for deciding when a new crate or package is ready to join the
  default pilot scope
- pointers to evidence from the first consumer rollout that justify the kit's
  defaults and cautions
- documented minimum environment assumptions that match the published template's
  actual runtime dependencies

#### Scenario: Preparing a second downstream rollout

- GIVEN a maintainer is onboarding a second consumer after Aspen
- WHEN they follow the integration kit
- THEN they can choose a starter profile, wire a working check path, and add
  consumer-local tuning without rediscovering Aspen's setup work

#### Scenario: Reviewing a consumer wrapper

- GIVEN a consumer uses a nix-managed or otherwise nonstandard nightly toolchain
- WHEN a reviewer checks the documented integration kit
- THEN they can see the expected wrapper responsibilities and minimum
  environment assumptions

### Requirement: High-Noise Lint Promotion Playbook

The project MUST maintain a promotion playbook for lint families that proved
too noisy for blind rollout in Aspen.

The playbook MUST:

- name `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, and
  `unbounded_loop` as inventory-first families
- require a per-crate or per-package inventory before a family is promoted
  into the default downstream path
- record known noise or classification patterns that reviewers should sort
  before deciding whether a finding is real debt, justified shell code, or a
  lint-precision problem
- define the evidence required before promoting a family into a later rollout
  phase
- reject blanket `allow`-based promotion as a substitute for triage

#### Scenario: Promoting `no_unwrap`

- GIVEN a maintainer wants to add `no_unwrap` to a downstream default profile
- WHEN they follow the playbook
- THEN they first build an inventory, classify the highest-risk production
  sites, and capture evidence for the reduced baseline before promotion

#### Scenario: Reviewing `unbounded_loop` findings

- GIVEN a downstream crate has many loop-related findings
- WHEN a reviewer follows the playbook
- THEN they can separate real unbounded retries from known event-loop patterns
  and identify any remaining lint-precision gaps explicitly

### Requirement: Audit-Ready Rollout Reconstruction Evidence

When the project claims a reusable downstream rollout kit is complete, it MUST
capture audit-ready evidence showing the documented reconstruction path works.

The evidence MUST include:

- a real end-to-end reconstruction transcript using the published templates or
  equivalent copied artifacts, not only syntax or help output
- the full helper scripts used for validation, stored as reviewable files
- verification notes that reference exact commands or stored helper scripts
  instead of elided here-doc summaries
- enough detail to show the documented starter path was reconstructed without
  relying on undocumented shell state or current downstream repo drift

#### Scenario: Reviewing Aspen phase-1 reconstruction

- GIVEN a reviewer inspects the rollout-kit verification artifacts
- WHEN they open the reconstruction evidence
- THEN they can see the exact helper scripts, the temp-worktree setup, and the
  transcript proving the published Aspen phase-1 path ran from the new templates

