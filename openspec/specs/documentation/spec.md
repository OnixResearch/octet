# Documentation Specification

## Purpose

Tiger Style documentation requirements for diagnostic wording, downstream
rollout guidance, namespace contracts, and consumer-facing rollout artifacts.
## Requirements
### Requirement: Standardized Diagnostic Structure

Every lint diagnostic MUST include:

1. A one-line warning message describing the violation
2. A help note in the exact form `Tiger Style (<principle>): <guidance>`;
   for lints with a concrete approved pattern, `<guidance>` MUST include an
   appended `Example: <approved pattern>` clause in the same help note
3. A note in the exact form `see Tiger Style guide: <section>`

#### Scenario: Developer sees lint for the first time

- GIVEN a developer runs tigerstyle and encounters `ambient_clock`
- WHEN they read the diagnostic
- THEN they see the principle name, the approved pattern, and where to
  read more without opening a separate document

#### Scenario: Diagnostic wording stays consistent across lints

- GIVEN two different tigerstyle lint diagnostics
- WHEN a developer compares their help and note lines
- THEN both diagnostics use the same `Tiger Style (...)` help prefix and
  the same `see Tiger Style guide: ...` note prefix

### Requirement: Canonical Tool-Scoped Tigerstyle Namespace

Every consumer-facing tigerstyle lint MUST be registered as a true tool lint in
canonical `tigerstyle::<lint_name>` form. Tool-scoped domain groups MUST be
registered against those true tool-lint IDs rather than ad-hoc plain-lint
aliases.

Standalone raw-rustc crates that use tool-scoped tigerstyle attrs MUST compile
on the supported nightly toolchain and opt into `#![feature(register_tool)]`
plus `#![register_tool(tigerstyle)]`.

#### Scenario: Tool-scoped per-lint attr resolves against a true tool lint

- GIVEN a crate on the supported nightly toolchain with
  `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN it uses a source-level attr like `#[allow(tigerstyle::no_unwrap)]`
- THEN rustc resolves that attr against the real `tigerstyle::no_unwrap` lint
  instead of a plain-lint compatibility shim

### Requirement: Tool-Lint Migration Precondition

Before converting all 25 consumer-facing lints, the project MUST validate the
planned migration mechanism on a single representative lint under the repo's
Dylint harness.

That precondition MUST prove both of these points together:

- `declare_tool_lint!(pub tigerstyle::...)` works for the representative lint
  under the project's Dylint loading path
- the intended bare compatibility mechanism — a one-member bare alias
  registered through `lint_store.register_group(...)` — still resolves the
  current bare config / lint-level path for that representative lint

#### Scenario: Representative spike proves canonical and compatibility paths

- GIVEN representative lint `no_unwrap`
- WHEN the project migrates only that lint to
  `declare_tool_lint!(pub tigerstyle::no_unwrap)` and registers a one-member
  bare compatibility alias for it
- THEN canonical tool-scoped attrs resolve in the supported standalone UI path
- AND bare `disabled_lints`, bare `[tigerstyle.lint_levels]`, and the current
  bare lint-level path still resolve for `no_unwrap`
- AND the full 25-lint migration does not proceed until that spike passes

### Requirement: Lint Groups By Spec Domain

The project MUST register lint groups matching spec domains:

- `tigerstyle::safety`
- `tigerstyle::naming`
- `tigerstyle::structure`
- `tigerstyle::purity`
- `tigerstyle::concurrency`

Consumers MUST be able to use group-level `#[allow]` and `#[deny]`
attributes when compiling on the supported nightly toolchain and opting into
`#![feature(register_tool)]` plus `#![register_tool(tigerstyle)]`.

#### Scenario: Suppressing a domain during phased rollout

- GIVEN a consumer in starter profile compiling on the supported nightly
  toolchain with `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- WHEN they add `#![allow(tigerstyle::safety)]` to a crate root
- THEN all safety-domain lints are suppressed for that crate

#### Scenario: Allowing one domain does not suppress another

- GIVEN a crate on the supported nightly toolchain with
  `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
- AND the crate would otherwise trigger both a safety lint and a naming lint
- WHEN it adds `#![allow(tigerstyle::safety)]` to the crate root
- THEN the safety lint is suppressed
- AND the naming lint still emits a diagnostic

#### Scenario: Escalating one domain during cleanup

- GIVEN a consumer wants stricter naming cleanup and is compiling on the
  supported nightly toolchain with `#![feature(register_tool)]` and
  `#![register_tool(tigerstyle)]`
- WHEN they add `#![deny(tigerstyle::naming)]` to a crate root
- THEN naming-domain diagnostics in that crate become hard errors

### Requirement: Per-Lint Configuration Support

The project MUST continue to support disabling individual lints and overriding
lint severity through the consumer-owned `dylint.toml` even after migrating the
canonical lint namespace to `tigerstyle::<lint_name>`.

The consumer-owned `[tigerstyle]` section in `dylint.toml` MUST keep these
behaviors:

- `disabled_lints` continues to use bare lint names like `no_unwrap`
- `[tigerstyle.lint_levels]` continues to use bare lint names like `no_unwrap`
- bare config names resolve to the matching canonical tool lint without
  requiring consumers to rename config keys to `tigerstyle::...`
- `disabled_lints` still prevents disabled lints from registering, so it beats
  level overrides and source-level attrs

#### Scenario: Bare config key still upgrades the canonical tool lint

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code after the tool-lint migration
- THEN the canonical `tigerstyle::no_unwrap` lint emits an error instead of a
  warning

#### Scenario: Bare disabled key still beats source-level attrs

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- AND source code requests a source-level lint attribute for
  `tigerstyle::no_unwrap`
- WHEN the lint suite runs on code that would otherwise violate `no_unwrap`
- THEN `no_unwrap` still emits no diagnostic

#### Scenario: Disabled bare key keeps the lint out of groups and aliases

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- WHEN the lint suite registers canonical domain groups and bare compatibility
  aliases
- THEN `no_unwrap` is omitted from those registrations
- AND no canonical group or bare compatibility path re-enables that lint

### Requirement: Rollout Kit Records New-Lint Aspen Classification

The downstream rollout kit MUST record Aspen evidence-backed classification for
newly added lint families before any starter-profile promotion.

The guidance MUST include:

- the per-lint evidence artifacts used for Aspen validation
- the classification of each pending lint family as `starter-candidate` or
  `inventory-first`
- the rule that inventory-first families stay disabled in starter-profile
  examples until a later promotion review lands
- any new lint-family names that consumers need to recognize during rollout

#### Scenario: Maintainer evaluates a newly added lint family

- GIVEN tigerstyle adds new warn-by-default lint families that are not yet in a
  starter rollout profile
- WHEN a maintainer follows the rollout kit
- THEN they can find the saved Aspen evidence, see the current classification
  for each family, and keep inventory-first families out of copied starter
  configs until later promotion work is complete

### Requirement: Rollout Kit Covers Low-Noise New-Lint Promotion

The downstream rollout kit MUST document how a lint family graduates from
`starter-candidate` classification into the copied starter profile.

The guidance MUST include:

- which low-noise new lint families are currently promoted into the starter
  profile
- which pending families remain inventory-first and therefore stay disabled in
  copied starter examples, README rollout text, and consumer template comments
- the documented starter package set that remains in scope for this promotion:
  `aspen-time`, `aspen-hlc`, and `aspen-core`
- saved Aspen evidence for the promoted starter profile, not only the earlier
  classification pass
- the first-class `cargo-tigerstyle` replay path used for that evidence
- deterministic evidence artifacts under the change `evidence/` directory:
  `replay-aspen-promoted-starter.sh`, `check-aspen-promoted-starter.py`,
  `aspen-promoted-starter-config.toml`,
  `aspen-promoted-starter-default.txt`,
  `aspen-promoted-starter-coordination.txt`, and
  `aspen-promoted-starter-counts.txt`
- separate saved proof for both the promoted starter scope and a focused
  `aspen-coordination` lib-only spotcheck that uses the same promoted lint set
- the exact promoted starter-profile configuration shape consumers should copy

#### Scenario: Maintainer copies the promoted starter profile

- GIVEN `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
  `negated_predicate` have already met the saved low-noise rule
- WHEN a maintainer copies the starter / boundary-first rollout example
- THEN those four families are enabled in that copied profile
- AND `unbounded_collection_growth` plus `too_many_parameters` remain disabled
  until a later promotion review lands

#### Scenario: Maintainer reviews promotion proof

- GIVEN a maintainer wants to confirm the promoted starter profile before
  copying it into a consumer workspace
- WHEN they follow the rollout kit
- THEN they can find the exact `cargo-tigerstyle` replay path
- AND separate saved transcripts for the promoted starter scope and the focused
  `aspen-coordination` lib-only spotcheck
- AND the checker/count artifact sits next to those transcripts under the
  change `evidence/` directory
- AND the proof still pins the starter package set to `aspen-time`,
  `aspen-hlc`, and `aspen-core`

