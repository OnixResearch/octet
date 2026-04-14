## Context

The current `Config` struct in `src/config.rs` has five threshold fields.
Lint registration in `src/lib.rs` unconditionally registers all 26 lints.
Consumers use source-level `#[allow]` or the rollout playbook's social
conventions to control which lints fire.

## Goals / Non-Goals

**Goals:** Let consumers enable/disable individual lints and set per-lint
severity through `dylint.toml`.

**Non-Goals:** Runtime lint selection via CLI flags. Dylint does not
support passing per-lint flags through cargo; config-file-based control
is the right approach.

## Decisions

### 1. `disabled_lints` list suppresses registration

**Choice:** Lints named in `disabled_lints` are not registered with the
lint store at all.

**Rationale:** Skipping registration entirely means no traversal cost for
disabled lints. Source-level `#[allow]` only suppresses diagnostics but
the lint pass still runs.

**Implementation:** In `register_lints`, check `cfg.disabled_lints`
before each `register_lints` + `register_*_pass` call pair.

**Alternative:** Register all lints and set their default level to `Allow`
— rejected because it still runs the pass and can produce confusing
"lint unknown" messages when mixed with source-level attributes.

### 2. `lint_levels` override default severity

**Choice:** A `[tigerstyle.lint_levels]` table maps lint names to
`"allow"`, `"warn"`, or `"deny"`.

**Rationale:** Consumers may want `compound_assertion` at `deny` level
(hard error) while keeping `ambient_clock` at `warn`. This is different
from disabling — the lint still runs but its findings have different
severity.

**Implementation:** After registration, iterate `lint_levels` and call
`lint_store.register_group_lints` with the appropriate level, or set
the lint's default level during registration.

**Alternative:** Rely on `#![warn(...)]` / `#![deny(...)]` in source —
rejected because it requires source changes in every crate.

### 3. Backward-compatible config schema

**Choice:** New fields use `#[serde(default)]` and are `Option` or
empty-collection defaults.

**Rationale:** Existing `dylint.toml` files with only threshold fields
must continue to work.

**Implementation:** `disabled_lints: Vec<String>` defaults to empty.
`lint_levels: HashMap<String, String>` defaults to empty.

### 4. Validation at load time

**Choice:** Unknown lint names in `disabled_lints` or `lint_levels`
produce a warning during config load, not a hard error.

**Rationale:** Forward-compatibility. A consumer may have config for
lints added in a newer tigerstyle version.

## Risks / Trade-offs

**[Config drift from source-level attrs]** → A lint disabled in
`dylint.toml` but `#[deny]`'d in source will silently win at the config
level (lint never registered, so the source attr is inert). Mitigation:
document that `dylint.toml` disabled_lints takes precedence over source
attributes.

**[Testing complexity]** → Need to test config parsing, lint-disable
behavior, and level overrides. Mitigation: integration tests with
different `dylint.toml` configs and expected diagnostic counts.
