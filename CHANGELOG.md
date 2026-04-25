# Changelog

Notable changes that affect consumers. Organized by date since consumers pin to
specific commits via `TIGERSTYLE_DYLINT_REV`.

For the full lint catalog, see `docs/lint-reference.md`.
For rollout profiles, see `docs/downstream-rollout.md`.

## 2026-04-16: Starter profile expanded

Promoted four low-noise lint families into the starter rollout profile:
`explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
`negated_predicate`.

Consumers using the starter `disabled_lints` list from `templates/dylint.toml`
will see these lints activate after updating. Evidence and classification data
are in `docs/downstream-rollout-evidence.md`.

## 2026-04-15: `todo!()` moved from `no_panic` to `no_todo`

`no_panic` no longer covers `todo!()`. A new `no_todo` lint owns `todo!()`
detection, and `unjustified_no_todo_allow` requires justification for
`allow(no_todo)` escapes.

**Migration:** consumers with justified `todo!()` stubs under `allow(no_panic)`
must migrate to `allow(no_todo)` or `allow(tigerstyle::no_todo)`.

## 2026-04-15: Tool-scoped lint attrs

Lint attrs migrated from bare names to tool-scoped canonical names:

- `#[allow(tigerstyle::no_unwrap)]` instead of `#[allow(no_unwrap)]`
- `#![deny(tigerstyle::naming)]` for domain groups

Bare names still work through compatibility aliases. Standalone raw-rustc
fixtures need `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`.

Config keys in `dylint.toml` stay bare (`no_unwrap`, not `tigerstyle::no_unwrap`).

## 2026-04-14: First-class runner replaces shell wrapper

`cargo tigerstyle check` and `nix run .#cargo-tigerstyle -- check` are the
supported consumer entry points. The historical shell wrapper is retained only
in archived replay evidence.

New consumers should use `[workspace.metadata.tigerstyle]` for runner policy
and `dylint.toml` for lint tuning. See `docs/consumer-guide.md`.

## 2026-04-14: Structured output

`cargo tigerstyle check` gained `--output-format` (json, sarif) and
`--artifact-dir` for machine-readable output and saved artifact bundles.

Exit codes: `0` clean/warning-only, `1` error-level findings, `2` integration
failures.

## 2026-04-13: Initial Aspen consumer rollout

First evidence-backed downstream rollout against the Aspen distributed systems
project. Established the starter rollout profile with `ambient_clock` and
`compound_assertion`. Evidence archived in
`openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`.
