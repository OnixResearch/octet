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

### Requirement: Rollout Kit Covers Post-Starter Package Widening

The downstream rollout kit MUST document how a consumer graduates a deferred
package into the default tigerstyle path after the starter rollout is green.

The guidance MUST include:

- the conditions for choosing one deferred package as the next promotion target
- the cleanup themes reviewers should classify before widening the default path
- a concrete widened-reference state derived from Aspen after `aspen-coordination`
  rejoined the default pilot
- evidence showing both the widened default pilot and the cleaned deferred
  package slice

#### Scenario: Promoting a deferred package after phase 1

- GIVEN a consumer finished the starter rollout and still has one deferred
  package with a bounded finding set
- WHEN a maintainer follows the rollout kit
- THEN they can inventory the slice, clean or justify the blockers, and prove
  the package can join the default path without rediscovering Aspen's phase-2
  promotion steps

### Requirement: Rollout Kit Has A Saved Non-Aspen Consumer Proof

The downstream rollout kit MUST include one saved proof that a non-Aspen Rust
consumer can follow the published templates and check-path guidance.

That proof MUST include:

- the chosen consumer workspace
- the copied `templates/tigerstyle-check.sh` and `templates/dylint.toml`
  workflow, or an exact saved helper that performs those copy/edit steps
- a saved transcript of the resulting tigerstyle check run
- enough sandbox or path-layout detail for a reviewer to replay the proof
  without Aspen-specific assumptions

#### Scenario: Reviewing the second consumer proof

- GIVEN a reviewer wants to know whether the rollout kit only works for Aspen
- WHEN they inspect the saved proof
- THEN they can replay the helper on a non-Aspen consumer and see the template-
  based tigerstyle check run end to end

### Requirement: Nix Check Path Exercises UI Lint Fixtures

The project MUST execute real Dylint UI fixtures as part of its nix-backed
validation path.

The nix check path MUST provide whatever rustup compatibility wiring the UI
harness needs, rather than silently skipping when `rustup` is absent.

The repository MUST keep a checked-in UI fixture whose expected stderr is
validated during that nix check path for every shipped tigerstyle lint.

The repository MUST keep checked-in config-override UI fixtures for every
configurable tigerstyle behavior:

- `function_length_max`
- `nesting_max_depth`
- `assertion_density_min_lines` / `assertion_density_min_assertions`
- `acronym_allowlist`

The UI harness MUST assert the expected compiler exit status for each fixture,
so deny-level fixtures still prove the command failed for the right reason
instead of passing on stderr text alone.

#### Scenario: Running nix flake check

- GIVEN a maintainer runs `nix flake check`
- WHEN the test suite reaches the UI harness
- THEN it runs checked-in Dylint UI fixtures instead of returning early because
  `rustup` is missing

#### Scenario: Lint-wide regression coverage

- GIVEN a shipped tigerstyle lint changes its diagnostic wording or stops firing
- WHEN the nix validation path runs
- THEN the matching checked-in fixture fails and reports the stderr mismatch

#### Scenario: Config-override regression coverage

- GIVEN a maintainer changes config parsing or a configurable threshold
- WHEN the nix validation path runs
- THEN the checked-in override fixtures catch regressions in the configured
  behavior

#### Scenario: Deny-level fixture exits non-zero

- GIVEN a fixture that intentionally triggers a deny-by-default lint such as
  `verified_purity`
- WHEN the UI harness runs it
- THEN the harness checks that the compiler command failed and that stderr
  matches the expected deny-level diagnostic

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
- any required sandbox, sibling-repo, or path-layout setup that the documented
  reconstruction path depends on
- archived text artifacts whose recorded commands reference the archived helper
  and evidence locations, not a deleted pre-archive change path
- if the documented reconstruction path is a saved helper command, a transcript
  whose top-level replay command matches that helper path exactly
- if the saved transcript starts with a command preamble, the repo MUST store a
  documented helper or exact capture command that emits that preamble
- consistency checks that verify the documented capture method, the saved
  helper, and the saved transcript agree on that preamble source
- check helpers that assert every summary claim the docs or verification notes
  make about the replay output, including zero-warning claims

#### Scenario: Reviewing transcript preamble provenance

- GIVEN a reviewer opens a saved reconstruction transcript whose first line is a
  `$ bash ...` command
- WHEN they inspect the saved helper scripts and consistency checks
- THEN they can see which documented helper emitted that preamble and verify the
  transcript was captured through that same helper path

### Requirement: Per-Lint Configuration Support

The project MUST support disabling individual lints and overriding lint
severity through the consumer-owned `dylint.toml`.

The consumer-owned `[tigerstyle]` section in `dylint.toml` MUST support these
optional controls and behaviors:

- an optional `disabled_lints` list that prevents named lints from registering
- an optional `lint_levels` table that maps lint names to `allow`, `warn`, or
  `deny`
- omitted `disabled_lints` and `lint_levels` keys default to empty collections
  and preserve the existing default lint levels
- a warning (not error) for unknown lint names in config
- a warning (not error) for invalid `lint_levels` values, with the invalid
  override ignored
- a warning (not error) when a lint appears in both `disabled_lints` and
  `lint_levels`, with the level override ignored

#### Scenario: Omitted per-lint controls keep existing behavior

- GIVEN a `dylint.toml` without `disabled_lints` or `lint_levels`
- WHEN the lint suite loads config
- THEN both controls default to empty and lints keep their default levels

#### Scenario: Disabling a noisy lint

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- WHEN the lint suite runs
- THEN `no_unwrap` produces no diagnostics

#### Scenario: Unknown lint name warns and continues

- GIVEN `disabled_lints = ["not_a_real_lint"]` in `dylint.toml`
- WHEN the lint suite loads config
- THEN it emits a warning and continues with the remaining config

#### Scenario: Invalid lint level warns and keeps default

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "fatal"` in `dylint.toml`
- WHEN the lint suite loads config
- THEN it emits a warning and `no_unwrap` keeps its default level

#### Scenario: Disabled lint beats level override

- GIVEN `disabled_lints = ["no_unwrap"]` and `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN the lint suite loads config and runs
- THEN it emits a warning about the conflicting config and `no_unwrap` stays disabled

#### Scenario: Allow level suppresses diagnostics

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "allow"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code
- THEN it emits no diagnostic

#### Scenario: Warn level keeps warning behavior

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "warn"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code
- THEN it emits a warning

#### Scenario: Deny level upgrades to error

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code
- THEN it emits an error instead of a warning

#### Scenario: Disabled lint beats source-level attrs

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- AND source code requests a source-level lint attribute for `tigerstyle::no_unwrap`
- WHEN the lint suite runs on code that would otherwise violate `no_unwrap`
- THEN `no_unwrap` still emits no diagnostic

