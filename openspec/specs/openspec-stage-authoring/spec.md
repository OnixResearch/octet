# OpenSpec Stage Authoring Specification

## Purpose

Define tigerstyle's repo-local OpenSpec proposal-stage, delta-spec-stage,
and design-stage authoring contract: copyable proposal, delta-spec, and
design templates with exact heading inventories, explicit repo-relative
artifact planning, and contributor/config guidance that points maintainers at
those templates before implementation.

## Requirements

### Requirement: Proposal Template Captures Scope, Artifacts, And Ordering

The system MUST ship a copyable OpenSpec proposal template at
`openspec/templates/proposal.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings:

- `## Why`
- `## What Changes`
- `## Non-Goals`
- `## Capabilities`
- `## Impact`

That template MUST also keep these exact prompt lines:

- `### New Capabilities`
- `### Modified Capabilities`
- `- **Files / artifacts**: <exact repo-relative paths touched by this change>`
- `- **Ordering / dependencies**: <what must land first or stay in lockstep>`
- `- **Verification / evidence**: <saved artifacts or helper/gate runs planned for review>`

#### Scenario: Maintainer names exact proposal surfaces before drafting specs

- GIVEN a maintainer copies `openspec/templates/proposal.md`
- WHEN they draft a new change proposal
- THEN they can name exact repo-relative files or artifacts touched by the change
- AND they can record ordering or dependency notes before implementation starts
- AND they can name the planned saved evidence or helper/gate runs for review

### Requirement: Design Template Captures Ordered Proof Planning

The system MUST ship a copyable OpenSpec design template at
`openspec/templates/design.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings:

- `## Context`
- `## Goals / Non-Goals`
- `## Decisions`
- `## Deterministic Proof Plan`
- `## Risks / Trade-offs`

Inside `## Deterministic Proof Plan`, the template MUST keep these exact prompt
labels:

- `- scenario or requirement:`
- `- artifact path:`
- `- command / helper:`
- `- ordering / prerequisites:`
- `- cleanup / postcondition:`

#### Scenario: Maintainer binds each proof artifact to path and order

- GIVEN a maintainer copies `openspec/templates/design.md`
- WHEN they plan proof for a new change
- THEN they can bind each scenario or requirement to a saved artifact path
- AND they can name the command or helper that produces that artifact
- AND they can record prerequisites, sequencing, and cleanup expectations

### Requirement: Delta-Spec Template Captures Normative Delta Shape

The system MUST ship a copyable OpenSpec delta-spec template at
`openspec/templates/spec.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings:

- `## ADDED Requirements`
- `## MODIFIED Requirements`
- `## REMOVED Requirements`

That template MUST also keep these exact prompt lines:

- `### Requirement: <requirement heading>`
- `The system MUST <normative behavior>`
- `#### Scenario: <scenario heading>`
- `- GIVEN <context>`
- `- WHEN <action>`
- `- THEN <observable outcome>`
- `This requirement is removed because <reason or replacement>.`

#### Scenario: Maintainer drafts a delta spec without inventing headings

- GIVEN a maintainer copies `openspec/templates/spec.md`
- WHEN they draft a change-local delta spec under `openspec/changes/<change>/specs/<domain>/spec.md`
- THEN they can add or modify requirements without inventing extra second-level headings
- AND they can start each added or modified requirement from an explicit `The system MUST ...` sentence
- AND they can add a named `#### Scenario:` block with `GIVEN` / `WHEN` / `THEN` steps

### Requirement: Delta-Spec Template Carries Repo-Specific Guardrails

The system MUST keep these exact repo-specific guidance comments in
`openspec/templates/spec.md`:

- `<!-- Copy this file into openspec/changes/<change>/specs/<domain>/spec.md and replace the placeholders. -->`
- `<!-- Change-local specs stay in delta form. Main specs under openspec/specs/<domain>/spec.md stay in full spec form. -->`
- `<!-- Start each added or modified requirement with an explicit sentence beginning "The system MUST ...". -->`
- `<!-- Give every added or modified requirement at least one "#### Scenario:" block. -->`
- `<!-- If acceptance depends on a saved artifact, describe the artifact's exact expected output or schema in the requirement body, not only the path. -->`

#### Scenario: Maintainer sees repo-specific delta-spec guardrails in the template

- GIVEN a maintainer opens `openspec/templates/spec.md`
- WHEN they read the template comments before drafting a delta spec
- THEN they can see that active-change specs stay in delta form while main specs stay in full-spec form
- AND they can see that added or modified requirements begin with `The system MUST ...`
- AND they can see that each added or modified requirement needs at least one `#### Scenario:` block
- AND they can see that proof-artifact requirements must describe exact expected output or schema, not just a path

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST point proposal-stage, delta-spec-stage, and design-stage
authors at the repo-local OpenSpec template files before implementation.

`docs/contributor-guide.md` MUST contain these exact standalone markdown-link
lines:

- `[OpenSpec proposal template](../openspec/templates/proposal.md)`
- `[OpenSpec delta-spec template](../openspec/templates/spec.md)`
- `[OpenSpec design template](../openspec/templates/design.md)`

`openspec/config.yaml` MUST contain one proposal rule that names
`openspec/templates/proposal.md` and one proposal rule that names
`openspec/templates/spec.md` for delta-spec authoring, plus one design rule
that names `openspec/templates/design.md`.

#### Scenario: Contributor guide exposes the delta-spec template

- GIVEN a maintainer opens `docs/contributor-guide.md`
- WHEN they look for repo-local OpenSpec authoring guidance
- THEN they can find the standalone delta-spec-template link line above
- AND the same guide section still exposes the proposal-template and design-template link lines above

#### Scenario: Proposal-stage guidance names the delta-spec template

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal-stage prompts are prepared for a new change
- THEN one proposal rule names `openspec/templates/proposal.md`
- AND one proposal rule names `openspec/templates/spec.md`
- AND design guidance still names `openspec/templates/design.md`
