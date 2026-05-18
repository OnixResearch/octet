# Downstream rollout kit

Start with `docs/consumer-guide.md` if you need first-run setup, config split,
or the current supported entry points. This page is the rollout playbook for
staged adoption and promotion decisions.

For Aspen-backed evidence, historical replay data, and archived wrapper
documentation, see `downstream-rollout-evidence.md`.

## Active path: first-class consumer integration

Use the first-class engine, not a copied shell wrapper.

### Rust workspaces

1. add consumer-owned lint tuning in `dylint.toml`
2. add runner metadata in `[workspace.metadata.octet]` in the workspace
   `Cargo.toml`
3. run `cargo octet check`

Example metadata:

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

### Nix workspaces

Use the published flake app or helper instead of copying bash glue:

- `nix run path:/path/to/slotcar#cargo-octet -- check`
- consumer flake checks should call the same `cargo-octet` engine through
  `lib.mkConsumerCheck`

Repo-local helpers like `just slotcar` should stay thin and delegate to the
same command surface rather than reimplementing toolchain, lockfile, or Dylint
policy.

## Starter rollout profiles

### Smoke profile

Use this when toolchain plumbing is still unproven.

- scope: one owned production crate
- lints: `ambient_clock`, `compound_assertion`, `contradictory_time`, `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, `negated_predicate`
- cargo-check args: `--lib`
- deps: `--no-deps`
- exit condition: scripted check runs end to end with a finding set small enough
  to classify by hand

Why this works:

- it proves the Dylint wiring before you widen scope
- it keeps the starter path on families that already stayed <= 3 findings on
  Aspen's starter scope and <= 5 findings on `aspen-coordination`
- it still avoids the noisy inventory-first families that need a separate
  cleanup owner first

### Boundary-first profile

Use this for the first repeatable default path.

- scope: 2-4 production crates with clear owners and moderate churn
- lints: `ambient_clock`, `compound_assertion`, `contradictory_time`, `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, `negated_predicate`
- cargo-check args: `--lib`
- deps: `--no-deps`

Why this works:

- `ambient_clock` exposes real shell/core boundary problems without needing a
  huge cleanup wave first
- `compound_assertion` stays low-noise and usually fixes quickly
- `contradictory_time` catches the easy mistake of accepting time as input and
  still reading the clock locally
- the four promoted new families already proved low-noise on Aspen before they
  moved into starter guidance

For Aspen-backed evidence behind this profile, see
`downstream-rollout-evidence.md`.

### Inventory-only profile

Use this before promoting any noisy lint family into a default path.

- scope: one lint family at a time, one crate or package slice at a time
- goal: build an inventory, not enforce a workspace-wide default yet
- default families: `ignored_result`, `no_unwrap`, `no_panic`, `no_todo`,
  `unchecked_narrowing`, `unbounded_loop`

Why this works:

- owners can classify debt without reaching for blanket `allow`s
- false-positive and shell-boundary patterns get written down before rollout
- promotion decisions stay tied to measured baselines

## Profile-to-config mapping

All three rollout profiles now use the same first-class engine path and differ
in two knobs:

- scope in `[workspace.metadata.octet]` (`default_scope`) or in explicit
  `cargo octet check -p ...` overrides
- `disabled_lints` / `lint_levels` entries in consumer-owned `dylint.toml`

Per-lint config rules to preserve in every consumer example:

- omitted `disabled_lints` / `lint_levels` default to empty collections
- unknown lint names and invalid level values warn and are ignored
- `disabled_lints` wins over `lint_levels` and source-level lint attrs because
  disabled lints are not registered

### breaking-change: migrate `todo!()` escapes from `no_panic` to `no_todo`

Current slotcar contract:

- `no_todo` owns `todo!()` in production code
- `no_panic` now owns only `panic!()`, `unimplemented!()`, and
  `unreachable!()`
- old escape-hatch spellings `#[allow(no_panic)]` and
  `#[allow(slotcar::no_panic)]` no longer justify `todo!()`
- migrate justified stubs to `#[allow(no_todo)]` or
  `#[allow(slotcar::no_todo)]`

Live Aspen code still carries bare `#![allow(no_panic)]` crate attrs, so keep
that bare-source migration note in rollout checklists until consumers finish the
rename.

### Smoke or boundary-first config example

Smoke and boundary-first use the same lint config; the difference is scope.
Smoke keeps scope to one owned crate. Boundary-first widens scope to 2-4 owned
production crates.

```toml
[octet]
disabled_lints = [
  "function_length",
  "acronym_style",
  "numeric_units",
  "unbounded_loop",
  "compound_condition",
  "verified_purity",
  "unjustified_allow",
  "nested_conditionals",
  "assertion_density",
  "multi_lock_ordering",
  "unbounded_collection_growth",
  "sentinel_fallback",
  "too_many_parameters",
  "bool_naming",
  "ignored_result",
  "unchecked_narrowing",
  "unchecked_division",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "borrowed_argument_types",
  "deref_polymorphism",
  "clone_then_borrow",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_unwrap",
  "no_panic",
  "public_unsafe_api",
  "no_todo",
  "unjustified_no_todo_allow",
  "no_recursion",
]
```

That leaves `ambient_clock`, `compound_assertion`, `contradictory_time`,
`explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
`negated_predicate` enabled.

For the Aspen-backed classification table and promotion evidence, see
`downstream-rollout-evidence.md`.

### Inventory-only config example

Inventory-only means one deferred lint family at a time, one scope slice at a
time. Example: inventory only `no_unwrap` while keeping every other lint out of
that run.

```toml
[octet]
disabled_lints = [
  "compound_assertion",
  "function_length",
  "acronym_style",
  "numeric_units",
  "unbounded_loop",
  "compound_condition",
  "verified_purity",
  "unjustified_allow",
  "nested_conditionals",
  "assertion_density",
  "multi_lock_ordering",
  "explicit_defaults",
  "unbounded_channel",
  "unbounded_collection_growth",
  "sentinel_fallback",
  "usize_in_public_api",
  "too_many_parameters",
  "negated_predicate",
  "bool_naming",
  "ignored_result",
  "unchecked_narrowing",
  "unchecked_division",
  "contradictory_time",
  "ambient_clock",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "borrowed_argument_types",
  "deref_polymorphism",
  "clone_then_borrow",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_panic",
  "public_unsafe_api",
  "no_todo",
  "unjustified_no_todo_allow",
  "no_recursion",
]

[octet.lint_levels]
no_unwrap = "warn"
```

After a family is low-noise, promote it by deleting it from `disabled_lints`
and, if needed, raising it in `lint_levels`.

## Promotion checklist for adding crates or packages

Add a new crate or package to the default path only when all of these hold:

1. the documented `cargo octet check` / `nix run ...#cargo-octet`
   path runs without ad-hoc shell setup beyond the rollout notes
2. the candidate scope has a bounded warning set with a named owner
3. consumer-owned `dylint.toml` changes are recorded with rationale and a
   cleanup trigger
4. deferred noisy families stay inventory-only until their own promotion review
   is done
5. no crate- or module-wide `#![allow(slotcar::...)]` is hiding unresolved
   rollout debt
6. one targeted rerun proves the candidate scope still produces actionable
   signal

## Phase-2 widening playbook

Use this after the starter rollout is green and one deferred package has a
clear owner.

1. pick one deferred package, not a whole family of packages at once
2. inventory the package's current findings by theme before fixing anything
3. separate shell-boundary cleanups from true core-logic refactors
4. rerun the package by itself until starter-lint spans disappear or shrink to
   a named, reviewable exception set
5. only then widen the no-argument default path and save a transcript for both
   the widened default run and the cleaned package slice

Aspen phase-2 example:

- deferred package: `aspen-coordination`
- baseline inventory: 21 `ambient_clock`, 2 `compound_assertion`, 4 `no_panic`
- cleanup themes: central wall-clock boundary helper, split verified
  assertions, remove panic-family blockers
- widened result: default pilot now includes `aspen-coordination` and two extra
  low-noise safety lints (`ignored_result`, `no_unwrap`)

## Consumer-owned `dylint.toml`

Copy `templates/dylint.toml` into the consumer workspace root.

The template separates four things that should stay consumer-owned:

- threshold overrides such as `function_length_max` or `nesting_max_depth`
- allowlists such as `acronym_allowlist`
- per-lint rollout controls (`disabled_lints`, `lint_levels`)
- rationale comments that explain why the workspace differs from slotcar's
  defaults and what cleanup would remove the override

Rules for the per-lint controls:

- omitted `disabled_lints` and `lint_levels` default to empty collections
- unknown lint names and invalid level values warn and are ignored
- `disabled_lints` and `[octet.lint_levels]` keep bare lint-name keys such
  as `no_unwrap`; source attrs and group attrs use canonical tool names such as
  `slotcar::no_unwrap` and `slotcar::safety`
- `disabled_lints` takes precedence over `lint_levels` and source-level attrs
  because disabled lints never register

For Aspen-specific example values, see `downstream-rollout-evidence.md`.

## Standalone raw-rustc tool attrs

If a consumer writes tool-scoped slotcar attrs in standalone raw-rustc code
or repo-local UI fixtures, use the supported nightly toolchain plus:

```rust
#![feature(register_tool)]
#![register_tool(slotcar)]
```

Without that prelude, rustc rejects canonical attrs such as
`#[allow(slotcar::ambient_clock)]` before the lint can run.

## `ambient_clock` boundary playbook

Treat `ambient_clock` as a boundary-ownership question, not only a style rule.

### Pattern: timeout -> deadline conversion helper

Prefer this split:

```rust
fn deadline_from(now: Instant, timeout: Duration) -> Instant {
    now + timeout
}

#[allow(slotcar::ambient_clock, reason = "RPC boundary owns deadline capture")]
fn rpc_deadline(timeout: Duration) -> Instant {
    deadline_from(Instant::now(), timeout)
}
```

Use a narrow item-scoped allow only on the helper that owns time capture.

Do not scatter `Instant::now() + timeout` across core helpers. If multiple call
sites all read the clock directly, move the read into one boundary helper or
inject `now`.

### Pattern: elapsed-time measurement in worker or retry code

Ask who owns scheduling.

- if the function is core logic, pass `started_at`, `now`, or `elapsed` in as a
  parameter
- if the function is the shell-owned worker loop itself, a narrow item-scoped
  allow is fine with a reason that names the scheduling boundary
- if the function already accepts `now`, `elapsed`, `deadline`, or another time
  input, do not add another direct clock read; fix the contradiction instead

### Pattern: repeated `Instant::now()` checks inside wait loops

Shell-owned wait loops are acceptable only when the loop itself owns the retry
or polling boundary and the allow stays on that one function.

Good shape:

```rust
#[allow(slotcar::ambient_clock, reason = "poll loop owns wall-clock timeout")]
fn wait_until_ready(...) -> Result<()> {
    let started_at = Instant::now();
    loop {
        if started_at.elapsed() >= timeout {
            return Err(...);
        }
        ...
    }
}
```

Bad shape:

- helper modules full of repeated `Instant::now()` calls with no single owner
- crate-wide `#![allow(slotcar::ambient_clock)]`
- functions that already accept time input but still read the wall clock again

### Relationship to `contradictory_time`

If a function already accepts `now`, `timestamp`, `deadline`, `elapsed`, or a
unit-suffixed time input, fix the `contradictory_time` problem first. The right
move is almost always to use the parameter you already have.

## High-noise lint promotion playbook

These families stay inventory-first until the consumer has a classified,
reviewable baseline:

- `ignored_result`
- `no_unwrap`
- `no_panic`
- `no_todo`
- `unchecked_narrowing`
- `unbounded_loop`
- `unbounded_collection_growth`
- `too_many_parameters`

### Inventory expectations by family

| Family | Inventory buckets | Known classification patterns | Promotion evidence |
| --- | --- | --- | --- |
| `ignored_result` | propagate with `?`, explicit best-effort shell cleanup, lint-precision gaps | fire-and-forget send/log paths, shutdown cleanup, intentionally ignored infallible conversions | per-crate count, owners for remaining sites, rerun after replacing silent drops with propagation or explicit handling |
| `no_unwrap` | production panic path, startup/bootstrap assumption, generated code, test-only leakage | parsing trusted constants, one-shot process bootstrap, accidental production `expect` left from tests | inventory with highest-risk production sites fixed first, narrow allows justified, reduced baseline transcript |
| `no_panic` | `unimplemented!`, `panic!`, `unreachable!`, generated code | crash-only boundary shims, impossible states not yet encoded in types | categorized inventory, follow-up cleanup issue for each surviving abort site, rerun proving default scope stays bounded |
| `no_todo` | `todo!`, legacy `allow(no_panic)` stubs, generated code | temporary rollout stubs, unfinished migrations, stale bare/tool-scoped escape hatches | categorized inventory, rename any surviving `allow(no_panic)` stub escapes to `no_todo`, rerun proving the remaining stub set stays reviewable |
| `unchecked_narrowing` | proven bounds, protocol-validated ranges, FFI/bit-packing, real truncation risk | prior `assert!` or guard, external type contracts, local proof comments | inventory plus representative fixes to `try_from` or explicit guards, any surviving allow linked to proof/range rationale |
| `unbounded_loop` | event loop, cancellable worker, bounded retry disguised as `loop`, true runaway retry | `select!`/reactor loops, loops with shutdown channels, loops whose bound lives in surrounding scheduler | inventory that separates event loops from real liveness debt, rerun after adding visible bounds or documented boundary allows |

### Narrow allow rules

A narrow item-scoped `#[allow(slotcar::... , reason = "...")]` is acceptable
only when all of these are true:

- the item owns the shell boundary or the local proof obligation
- the reason names why this site is exceptional
- the allow stays on one function, impl item, or small module boundary
- the inventory still records the site so later cleanup can revisit it

Not acceptable:

- crate-wide or package-wide blanket suppression
- enabling a noisy family and then silencing whole modules to get a green run
- using an allow instead of building the first inventory
- hiding rollout debt in `clippy.toml`, `RUSTFLAGS`, or undocumented shell
  aliases

## Evidence and historical replay

For Aspen phase-1 reconstruction, phase-2 widening evidence, the historical
wrapper path, second consumer proof, and all archived replay helpers, see
`downstream-rollout-evidence.md`.
