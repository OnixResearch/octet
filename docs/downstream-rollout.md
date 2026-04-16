# Downstream rollout kit

Aspen gave `tigerstyle` its first evidence-backed downstream rollout. This kit
turns that rollout into guidance for later consumers.

## Active path: first-class consumer integration

Use the first-class engine, not a copied shell wrapper.

### Rust workspaces

1. add consumer-owned lint tuning in `dylint.toml`
2. add runner metadata in `[workspace.metadata.tigerstyle]` in the workspace
   `Cargo.toml`
3. run `cargo tigerstyle check`

Example metadata:

```toml
[workspace.metadata.tigerstyle]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

### Nix workspaces

Use the published flake app or helper instead of copying bash glue:

- `nix run path:/path/to/tigerstyle#cargo-tigerstyle -- check`
- consumer flake checks should call the same `cargo-tigerstyle` engine through
  `lib.mkConsumerCheck`

Repo-local helpers like `just tigerstyle` should stay thin and delegate to the
same command surface rather than reimplementing toolchain, lockfile, or Dylint
policy.

## Reference state: use the Aspen archive, not Aspen HEAD

Phase-1 reference state lives in the archived OpenSpec change:

- Aspen rollout commit: `909e5e9fa` (`tigerstyle: add phase-1 rollout path and follow-up changes`)
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
- rollout notes: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- default-run transcript: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- deferred-package spotcheck: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-coordination-spotcheck.txt`
- phase-1 counts: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
- wrapper + config snapshot: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`

Current Aspen workspace scripts may already include later packages and later lint
families. Reconstruct the starter path from the archived change above, not from
whatever Aspen HEAD happens to run today.

## Current widened reference: Aspen phase 2

Use phase 1 as the canonical starter path. After that, Aspen's current widened
reference is captured in this repo by:

- replay helper: `bash openspec/changes/aspen-phase2-widening/evidence/replay-aspen-phase2.sh`
- Aspen HEAD snapshot: `openspec/changes/aspen-phase2-widening/evidence/aspen-phase2-meta.txt`
- widened default pilot: `openspec/changes/aspen-phase2-widening/evidence/aspen-phase2-default-pilot.txt`
- targeted `aspen-coordination` rerun: `openspec/changes/aspen-phase2-widening/evidence/aspen-phase2-coordination.txt`
- widening assertions: `openspec/changes/aspen-phase2-widening/evidence/aspen-phase2-check.txt`

What that widened reference proves:

- default scope now includes `-p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination`
- Aspen's current default lint set is now `ambient_clock`,
  `compound_assertion`, `contradictory_time`, `ignored_result`, and
  `no_unwrap`
- a targeted `aspen-coordination` rerun no longer emits starter-lint spans

Treat this as the first post-starter promotion example, not as a replacement
for the archived phase-1 starter reconstruction.

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

Aspen starter references for this profile:

- historical phase-1 baseline: 6 warnings total, all `ambient_clock`
  - evidence: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- promoted starter replay on the same package set: 10 warnings total
  - 6 `ambient_clock`
  - 1 `explicit_defaults`
  - 3 `usize_in_public_api`
  - evidence: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-default.txt`

### Inventory-only profile

Use this before promoting any noisy lint family into a default path.

- scope: one lint family at a time, one crate or package slice at a time
- goal: build an inventory, not enforce a workspace-wide default yet
- default families: `ignored_result`, `no_unwrap`, `no_panic`,
  `unchecked_narrowing`, `unbounded_loop`

Why this works:

- owners can classify debt without reaching for blanket `allow`s
- false-positive and shell-boundary patterns get written down before rollout
- promotion decisions stay tied to measured baselines

## Profile-to-config mapping

All three rollout profiles now use the same first-class engine path and differ
in two knobs:

- scope in `[workspace.metadata.tigerstyle]` (`default_scope`) or in explicit
  `cargo tigerstyle check -p ...` overrides
- `disabled_lints` / `lint_levels` entries in consumer-owned `dylint.toml`

Per-lint config rules to preserve in every consumer example:

- omitted `disabled_lints` / `lint_levels` default to empty collections
- unknown lint names and invalid level values warn and are ignored
- `disabled_lints` wins over `lint_levels` and source-level lint attrs because
  disabled lints are not registered

### Smoke or boundary-first config example

Smoke and boundary-first use the same lint config; the difference is scope.
Smoke keeps scope to one owned crate. Boundary-first widens scope to 2-4 owned
production crates.

```toml
[tigerstyle]
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
  "too_many_parameters",
  "bool_naming",
  "ignored_result",
  "unchecked_narrowing",
  "unchecked_division",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_unwrap",
  "no_panic",
  "unjustified_no_panic_allow",
  "no_recursion",
]
```

That leaves `ambient_clock`, `compound_assertion`, `contradictory_time`,
`explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
`negated_predicate` enabled.

### Aspen classification and promoted starter families

Saved evidence now splits into two layers:

- archived classification pass for the six new families:
  - replay helper: `bash openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/replay-aspen-new-lints.sh`
  - summary table: `openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/validation-counts.txt`
- promoted starter replay for the copied starter profile:
  - replay helper: `bash openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/replay-aspen-promoted-starter.sh`
  - config snapshot: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-config.toml`
  - summary table: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-counts.txt`
  - saved spotcheck: `openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/aspen-promoted-starter-coordination.txt`

Low-noise rule for promotion stayed the same:

- `starter-candidate`: <= 3 findings on the starter scope and <= 5 findings on
  `aspen-coordination`
- `inventory-first`: anything larger stays out of starter-profile promotion for
  now

Current Aspen-backed starter status:

| Family | Starter scope | `aspen-coordination` | Starter status |
| --- | ---: | ---: | --- |
| `explicit_defaults` | 1 | 2 | `promoted` |
| `unbounded_channel` | 0 | 0 | `promoted` |
| `unbounded_collection_growth` | 0 | 8 | `inventory-first` |
| `usize_in_public_api` | 3 | 0 | `promoted` |
| `too_many_parameters` | 0 | 15 | `inventory-first` |
| `negated_predicate` | 0 | 2 | `promoted` |

The promoted starter replay stays pinned to the documented starter package set
from `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`:
`aspen-time`, `aspen-hlc`, and `aspen-core`.

Only `unbounded_collection_growth` and `too_many_parameters` still stay out of
the copied starter-profile config among these six families.

### Inventory-only config example

Inventory-only means one deferred lint family at a time, one scope slice at a
time. Example: inventory only `no_unwrap` while keeping every other lint out of
that run.

```toml
[tigerstyle]
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
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_panic",
  "unjustified_no_panic_allow",
  "no_recursion",
]

[tigerstyle.lint_levels]
no_unwrap = "warn"
```

After a family is low-noise, promote it by deleting it from `disabled_lints`
and, if needed, raising it in `lint_levels`.

## Promotion checklist for adding crates or packages

Add a new crate or package to the default path only when all of these hold:

1. the documented `cargo tigerstyle check` / `nix run ...#cargo-tigerstyle`
   path runs without ad-hoc shell setup beyond the rollout notes
2. the candidate scope has a bounded warning set with a named owner
3. consumer-owned `dylint.toml` changes are recorded with rationale and a
   cleanup trigger
4. deferred noisy families stay inventory-only until their own promotion review
   is done
5. no crate- or module-wide `#![allow(tigerstyle::...)]` is hiding unresolved
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

## Historical wrapper path for replay only

The copied shell-wrapper path is retained only inside archived replay materials
and historical rollout reconstruction. It is no longer the active recommended
consumer path.

For historical replay, use the archived reconstruction helpers directly instead
of copying a live repo wrapper.

That historical wrapper owned these responsibilities:

- derives a full nightly toolchain triple from `tigerstyle/rust-toolchain.toml`
  and the host triple
- wraps `rustup show active-toolchain` and `rustup which rustc` when the local
  toolchain manager reports something nonstandard like `nix-managed`
- wraps `cargo` and `dylint-link` so `RUSTUP_TOOLCHAIN` is set during build and
  link steps
- bootstraps an optional pinned `dylint_driver` in an isolated temporary crate
  so the consumer workspace `Cargo.lock` stays untouched
- restores the original `Cargo.lock` and fails if the check path still mutates
  the consumer lockfile
- points Dylint at a local `tigerstyle` checkout through `TIGERSTYLE_PATH` or a
  sibling checkout
- owns the documented starter profile, default scope, and default cargo-check
  args in one reviewable script

The starter-profile lint list in the template is documentation and review
scoping, not a hard per-lint CLI filter. External Dylint runs still use the
consumer's enabled warn-by-default tigerstyle lints unless the consumer changes
source-level lint configuration.

Minimum assumptions for the template:

- `bash`, `cargo`, `rustc`, `python3`, `cargo-dylint`, `dylint-link`, and
  `git` are on `PATH`
- standard Unix userland is available for `awk`, `basename`, `chmod`, `cmp`,
  `cp`, `ln`, `mkdir`, `mktemp`, and `sed`
- the consumer repo has its own `dylint.toml`
- a local `tigerstyle` checkout exists next to the consumer repo, or
  `TIGERSTYLE_PATH` is set
- the maintainer edits `default_scope` before landing the script

When wrappers are unnecessary, delete the wrapper section and call plain
`cargo dylint`. The template is a recipe, not a policy requirement.

## Second consumer proof

Non-Aspen replay proof in this repo:

- replay helper: `bash openspec/changes/second-consumer-proof/evidence/replay-second-consumer.sh`
- consumer snapshot: `openspec/changes/second-consumer-proof/evidence/second-consumer-meta.txt`
- run transcript: `openspec/changes/second-consumer-proof/evidence/second-consumer-proof.txt`
- replay assertions: `openspec/changes/second-consumer-proof/evidence/second-consumer-check.txt`

Current saved proof uses `aspen-verus-metrics` from a disposable sandbox. That
archived helper still copies the historical shell wrapper plus
`templates/dylint.toml`, edits `default_scope` to `-p aspen-verus-metrics`, then
runs the copied script through a tigerstyle-provided nightly toolchain
environment. Keep this as replay evidence only; new consumers should use
`cargo tigerstyle check` plus `[workspace.metadata.tigerstyle]`.

### Aspen wrapper facts worth preserving

Aspen phase 1 needed three wrapper behaviors:

1. `rustup show active-toolchain` had to return a real nightly triple instead of
   `nix-managed`
2. `dylint-link` needed `RUSTUP_TOOLCHAIN` during link steps
3. `dylint_driver` needed an unreleased upstream fix; the current template
   satisfies that by bootstrapping a pinned driver in an isolated temporary
   crate before touching the consumer workspace

That third point is exactly why the template exposes `TIGERSTYLE_DYLINT_REV`
instead of baking Aspen's patch revision into `tigerstyle` itself.

## Consumer-owned `dylint.toml`

Copy `templates/dylint.toml` into the consumer workspace root.

The template separates four things that should stay consumer-owned:

- threshold overrides such as `function_length_max` or `nesting_max_depth`
- allowlists such as `acronym_allowlist`
- per-lint rollout controls (`disabled_lints`, `lint_levels`)
- rationale comments that explain why the workspace differs from tigerstyle's
  defaults and what cleanup would remove the override

Rules for the per-lint controls:

- omitted `disabled_lints` and `lint_levels` default to empty collections
- unknown lint names and invalid level values warn and are ignored
- `disabled_lints` and `[tigerstyle.lint_levels]` keep bare lint-name keys such
  as `no_unwrap`; source attrs and group attrs use canonical tool names such as
  `tigerstyle::no_unwrap` and `tigerstyle::safety`
- `disabled_lints` takes precedence over `lint_levels` and source-level attrs
  because disabled lints never register

Aspen phase-1 example values were:

- `function_length_max = 100`
- `nesting_max_depth = 4`
- `assertion_density_min_lines = 30`
- `assertion_density_min_assertions = 2`
- `acronym_allowlist = ["IO"]`

Those values belong in Aspen's rollout notes and `dylint.toml`, not in
`tigerstyle` defaults.

Evidence:

- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`

## Standalone raw-rustc tool attrs

If a consumer writes tool-scoped tigerstyle attrs in standalone raw-rustc code
or repo-local UI fixtures, use the supported nightly toolchain plus:

```rust
#![feature(register_tool)]
#![register_tool(tigerstyle)]
```

Without that prelude, rustc rejects canonical attrs such as
`#[allow(tigerstyle::ambient_clock)]` before the lint can run.

## `ambient_clock` boundary playbook

Treat `ambient_clock` as a boundary-ownership question, not only a style rule.

### Pattern: timeout -> deadline conversion helper

Prefer this split:

```rust
fn deadline_from(now: Instant, timeout: Duration) -> Instant {
    now + timeout
}

#[allow(tigerstyle::ambient_clock, reason = "RPC boundary owns deadline capture")]
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
#[allow(tigerstyle::ambient_clock, reason = "poll loop owns wall-clock timeout")]
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
- crate-wide `#![allow(tigerstyle::ambient_clock)]`
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
- `unchecked_narrowing`
- `unbounded_loop`
- `unbounded_collection_growth`
- `too_many_parameters`

### Inventory expectations by family

| Family | Inventory buckets | Known classification patterns | Promotion evidence |
| --- | --- | --- | --- |
| `ignored_result` | propagate with `?`, explicit best-effort shell cleanup, lint-precision gaps | fire-and-forget send/log paths, shutdown cleanup, intentionally ignored infallible conversions | per-crate count, owners for remaining sites, rerun after replacing silent drops with propagation or explicit handling |
| `no_unwrap` | production panic path, startup/bootstrap assumption, generated code, test-only leakage | parsing trusted constants, one-shot process bootstrap, accidental production `expect` left from tests | inventory with highest-risk production sites fixed first, narrow allows justified, reduced baseline transcript |
| `no_panic` | `todo!`, `unimplemented!`, `panic!`, `unreachable!`, generated code | legacy stubs, crash-only boundary shims, impossible states not yet encoded in types | categorized inventory, follow-up cleanup issue for each surviving allow, rerun proving default scope stays bounded |
| `unchecked_narrowing` | proven bounds, protocol-validated ranges, FFI/bit-packing, real truncation risk | prior `assert!` or guard, external type contracts, local proof comments | inventory plus representative fixes to `try_from` or explicit guards, any surviving allow linked to proof/range rationale |
| `unbounded_loop` | event loop, cancellable worker, bounded retry disguised as `loop`, true runaway retry | `select!`/reactor loops, loops with shutdown channels, loops whose bound lives in surrounding scheduler | inventory that separates event loops from real liveness debt, rerun after adding visible bounds or documented boundary allows |

### Narrow allow rules

A narrow item-scoped `#[allow(tigerstyle::... , reason = "...")]` is acceptable
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

## Aspen phase-1 reconstruction

Exact saved-transcript replay path from this repo:

```bash
bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh
```

That archived capture helper writes
`openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.txt`.
It prints the same top-level command preamble saved in that transcript, then
invokes the replay helper below. The replay helper performs these steps
internally:

1. create a disposable sandbox parent with Aspen's required sibling layout:
   - `aspen/` worktree at commit `909e5e9fa`
   - `aspen-dns -> /path/to/aspen-dns`
   - `aspen-wasm-plugin -> /path/to/aspen-wasm-plugin`
2. copy the archived phase-1 shell wrapper together with `templates/dylint.toml`
   into the Aspen sandbox checkout
3. configure the Aspen phase-1 scope:
   - `-p aspen-time`
   - `-p aspen-hlc`
   - `-p aspen-core`
4. apply the archived Aspen phase-1 `dylint.toml` values
5. export `TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9`
   while running the default path and the `aspen-coordination` spotcheck
6. confirm the archived phase-1 result:
   - 6 warnings total
   - `aspen-time`: 2 `ambient_clock`
   - `aspen-core`: 4 `ambient_clock`
   - `aspen-hlc`: 0 starter findings
7. confirm the archived defer decision:
   - 21 `ambient_clock`
   - 2 `compound_assertion`
   - 4 `no_panic`

Aspen needs the sibling entries because path dependencies in `aspen-dns` and
`aspen-wasm-plugin` resolve back through `../aspen`. The helper also removes the
sandbox worktree on exit.

Archive references for that reconstruction:

- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-coordination-spotcheck.txt`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
- `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`
