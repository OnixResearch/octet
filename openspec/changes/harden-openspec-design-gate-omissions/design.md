## Context

Tigerstyle already hardened proposal, delta-spec, and task authoring shape, but
`review_metrics promotions min=3` still shows one design-stage omission class
far above the rest: 52 repeated findings where designs fail to say which
scenario a decision covers, fail to name negative controls or continued-warning
boundaries, or fail to state what exact output makes a saved artifact pass.

The current `openspec/templates/design.md` preserves section order and a coarse
proof-plan skeleton, but it does not force decision-to-scenario traceability or
exact acceptance-signal planning. `openspec/config.yaml` currently tells authors
to follow the design template, yet it does not say that every added/modified
scenario heading or zero-scenario requirement must be covered explicitly.

## Goals / Non-Goals

**Goals:**
- make each design decision name the exact scenario or requirement headings it covers
- make boundary cases and continued-warning expectations explicit before review
- make saved proof artifacts state an exact expected output or acceptance signal
- harden design-stage guidance in `openspec/config.yaml` so the gate prompt asks for the same contract the template encodes

**Non-Goals:**
- changing proposal-stage or tasks-stage governance in this change
- retrofitting old archived designs to the new prompt shape
- promising immediate review-metric improvement before future changes use the new contract

## Decisions

### 1. Decision blocks carry explicit coverage and boundary prompts

**Choice:** extend each design-template decision block with exact `**Covers:**`
and `**Boundary cases:**` prompts.

**Rationale:** the highest-repeat design omissions are missing scenario
coverage and missing non-trigger boundaries. Putting those prompts inside every
decision block makes the reviewer-visible gap obvious before implementation.

**Alternative considered:** keep free-form decision prose and rely on reviewers
to ask for the missing mapping. Rejected because that is the churn this change
is trying to remove.

**Implementation:** update `openspec/templates/design.md` and the
`openspec-stage-authoring` delta spec so every copied decision block includes:

- `**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>`
- `**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>`

When a delta requirement has one or more scenarios, `**Covers:**` names the
exact scenario headings. A bare requirement heading is only valid for a
zero-scenario requirement.

### 2. Deterministic proof items carry exact acceptance signals

**Choice:** add one exact proof-plan prompt line,
`- exact expected output / acceptance signal:`, alongside the existing path,
command, ordering, and cleanup fields.

**Rationale:** repeated design findings show that path-only artifact planning is
not enough; review needs the exact line set, schema, or pass condition that the
artifact must prove.

**Alternative considered:** rely on the existing `artifact path` and
`cleanup / postcondition` lines. Rejected because neither field says what exact
output makes the artifact acceptable.

**Implementation:** update `openspec/templates/design.md` and the delta spec so
`## Deterministic Proof Plan` keeps the new prompt line in addition to the
existing required labels.

### 3. Design-stage guidance names full coverage obligations

**Choice:** harden the `openspec/config.yaml` design rule so it names
`openspec/templates/design.md`, requires every added/modified scenario heading
or zero-scenario requirement to appear in design coverage, and requires named
boundary cases plus exact acceptance signals when saved proof artifacts matter.

**Rationale:** template changes alone do not help if stage guidance still asks
for only a generic design. The repo-local gate prompt must ask for the same
coverage and artifact contract.

**Alternative considered:** link the template path only. Rejected because the
existing design rule already does that and still leaves repeated omissions.

**Implementation:** replace the current minimal design rule text in
`openspec/config.yaml` with one rule that keeps the template path and spells out
the new coverage and boundary expectations. The exact-five-`##`-headings rule
applies to the shipped template file `openspec/templates/design.md`, not to
every authored change-local `design.md` copied from it. `docs/contributor-guide.md`
keeps its existing template links unchanged in this rollout; alignment is
retained, not re-authored.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Maintainer maps design decisions to exact delta-spec coverage`
- artifact path:
  - `openspec/changes/harden-openspec-design-gate-omissions/evidence/design-template-coverage-fields.txt`
- command / helper:
  - `python3` extract over `openspec/templates/design.md`
- exact expected output / acceptance signal:
  - the saved extract prints these exact lines, in order:
    - `## Context`
    - `## Goals / Non-Goals`
    - `## Decisions`
    - `## Deterministic Proof Plan`
    - `## Risks / Trade-offs`
    - `**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>`
    - `**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>`
- ordering / prerequisites:
  - update `openspec/templates/design.md` first so the extract matches final text
- cleanup / postcondition:
  - no temp files remain outside the saved artifact

- scenario or requirement:
  - `Scenario: Maintainer names negative controls and proof acceptance signals`
- artifact path:
  - `openspec/changes/harden-openspec-design-gate-omissions/evidence/design-template-proof-fields.txt`
- command / helper:
  - `python3` extract over `openspec/templates/design.md`
- exact expected output / acceptance signal:
  - the saved extract prints these exact lines, in order:
    - `## Deterministic Proof Plan`
    - `- scenario or requirement:`
    - `- artifact path:`
    - `- command / helper:`
    - `- exact expected output / acceptance signal:`
    - `- ordering / prerequisites:`
    - `- cleanup / postcondition:`
- ordering / prerequisites:
  - update the proof-plan prompt block before saving the extract
- cleanup / postcondition:
  - the artifact reflects the final template text only

- scenario or requirement:
  - `Scenario: Design-stage guidance names coverage and boundary expectations`
- artifact path:
  - `openspec/changes/harden-openspec-design-gate-omissions/evidence/design-stage-guidance.txt`
- command / helper:
  - `python3` extract over `openspec/config.yaml`
- exact expected output / acceptance signal:
  - the saved extract prints one final design-rule excerpt that contains all of:
    - `openspec/templates/design.md`
    - `every added or modified scenario heading or zero-scenario requirement`
    - `boundary cases`
    - `exact expected output or acceptance signal`
- ordering / prerequisites:
  - update `openspec/config.yaml` after the template wording is final so the rule matches the template
- cleanup / postcondition:
  - the artifact contains only the final guidance line needed for review

- scenario or requirement:
  - human/oracle checkpoint for the full change
- artifact path:
  - `openspec/changes/harden-openspec-design-gate-omissions/evidence/design-gate.txt`
- command / helper:
  - `openspec_gate stage=design change=harden-openspec-design-gate-omissions`
- exact expected output / acceptance signal:
  - the transcript reports no missing design-coverage, boundary-case, or artifact-contract findings for this change
- ordering / prerequisites:
  - run after the final proposal, delta spec, design, and tasks are in place
- cleanup / postcondition:
  - the saved transcript references the active change path and needs no post-run cleanup

## Risks / Trade-offs

- more template ceremony -> keep the new prompts to the exact repeated omission classes instead of adding broad prose sections
- over-constraining simple changes -> let authors list one requirement heading in `**Covers:**` when a change has no scenario split
- drift between template and config guidance -> verify both with saved extracts and a design-stage gate transcript
