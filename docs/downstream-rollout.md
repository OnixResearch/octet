# Downstream rollout kit

Aspen gave `tigerstyle` its first evidence-backed downstream rollout. This kit
turns that rollout into copyable guidance for later consumers.

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

## Starter rollout profiles

### Smoke profile

Use this when toolchain plumbing is still unproven.

- scope: one owned production crate
- lints: `ambient_clock`, `compound_assertion`, `contradictory_time`
- cargo-check args: `--lib`
- deps: `--no-deps`
- exit condition: scripted check runs end to end with a finding set small enough
  to classify by hand

Why this works:

- it proves the Dylint wiring before you widen scope
- it avoids dropping into noisy families before the wrapper/config story is
  stable

### Boundary-first profile

Use this for the first repeatable default path.

- scope: 2-4 production crates with clear owners and moderate churn
- lints: `ambient_clock`, `compound_assertion`, `contradictory_time`
- cargo-check args: `--lib`
- deps: `--no-deps`

Why this works:

- `ambient_clock` exposes real shell/core boundary problems without needing a
  huge cleanup wave first
- `compound_assertion` stays low-noise and usually fixes quickly
- `contradictory_time` catches the easy mistake of accepting time as input and
  still reading the clock locally

Aspen phase-1 reference for this profile:

- scope: `-p aspen-time -p aspen-hlc -p aspen-core`
- result: 6 warnings total, all `ambient_clock`
- evidence: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`

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

## Promotion checklist for adding crates or packages

Add a new crate or package to the default path only when all of these hold:

1. the scripted check runs without ad-hoc shell setup beyond the documented
   wrapper/template
2. the candidate scope has a bounded warning set with a named owner
3. consumer-owned `dylint.toml` changes are recorded with rationale and a
   cleanup trigger
4. deferred noisy families stay inventory-only until their own promotion review
   is done
5. no crate- or module-wide `#![allow(tigerstyle::...)]` is hiding unresolved
   rollout debt
6. one targeted rerun proves the candidate scope still produces actionable
   signal

## Check-path recipe for nonstandard toolchains

Copy `templates/tigerstyle-check.sh` into the consumer repo when plain
`cargo dylint` is not reliable enough yet.

```bash
cp templates/tigerstyle-check.sh /path/to/consumer/scripts/tigerstyle-check.sh
chmod +x /path/to/consumer/scripts/tigerstyle-check.sh
```

The template owns these responsibilities:

- derives a full nightly toolchain triple from `tigerstyle/rust-toolchain.toml`
  and the host triple
- wraps `rustup show active-toolchain` and `rustup which rustc` when the local
  toolchain manager reports something nonstandard like `nix-managed`
- wraps `cargo` and `dylint-link` so `RUSTUP_TOOLCHAIN` is set during build and
  link steps
- keeps optional `dylint_driver` patching in a temporary `CARGO_HOME` instead of
  mutating the caller's shell
- points Dylint at a local `tigerstyle` checkout through `TIGERSTYLE_PATH` or a
  sibling checkout
- owns the documented starter profile, default scope, and default cargo-check
  args in one reviewable script

Minimum assumptions for the template:

- `bash`, `cargo`, `rustc`, `python3`, `cargo-dylint`, `dylint-link`, and
  `git` are on `PATH`
- standard Unix userland is available for `awk`, `basename`, `chmod`, `ln`,
  `mkdir`, `mktemp`, and `sed`
- the consumer repo has its own `dylint.toml`
- a local `tigerstyle` checkout exists next to the consumer repo, or
  `TIGERSTYLE_PATH` is set
- the maintainer edits `default_scope` before landing the script

When wrappers are unnecessary, delete the wrapper section and call plain
`cargo dylint`. The template is a recipe, not a policy requirement.

### Aspen wrapper facts worth preserving

Aspen phase 1 needed three wrapper behaviors:

1. `rustup show active-toolchain` had to return a real nightly triple instead of
   `nix-managed`
2. `dylint-link` needed `RUSTUP_TOOLCHAIN` during link steps
3. `dylint_driver` needed an unreleased upstream fix, injected through a
   temporary `CARGO_HOME/config.toml`

That third point is exactly why the template exposes `TIGERSTYLE_DYLINT_REV`
instead of baking Aspen's patch revision into `tigerstyle` itself.

## Consumer-owned `dylint.toml`

Copy `templates/dylint.toml` into the consumer workspace root.

The template separates three things that should stay consumer-owned:

- threshold overrides such as `function_length_max` or `nesting_max_depth`
- allowlists such as `acronym_allowlist`
- rationale comments that explain why the workspace differs from tigerstyle's
  defaults and what cleanup would remove the override

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

A maintainer should be able to reconstruct Aspen's starter path from this repo
alone:

1. create a disposable sandbox parent that recreates Aspen's sibling-repo
   layout before checking out the archived phase-1 commit:

   ```bash
   sandbox=$(mktemp -d /tmp/aspen-phase1-sandbox.XXXXXX)
   ln -s /path/to/aspen-dns "$sandbox/aspen-dns"
   ln -s /path/to/aspen-wasm-plugin "$sandbox/aspen-wasm-plugin"
   git -C /path/to/aspen worktree add --detach "$sandbox/aspen" 909e5e9fa
   cd "$sandbox/aspen"
   ```

   Aspen needs those sibling entries because path dependencies in
   `aspen-dns` and `aspen-wasm-plugin` resolve back through `../aspen`.

2. copy `templates/tigerstyle-check.sh` and `templates/dylint.toml`
3. keep the boundary-first lint set:
   - `ambient_clock`
   - `compound_assertion`
   - `contradictory_time`
4. set `default_scope` to:
   - `-p aspen-time`
   - `-p aspen-hlc`
   - `-p aspen-core`
5. if matching Aspen's old nix-managed nightly problem, export:

   ```bash
   export TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9
   ```

6. run:

   ```bash
   scripts/tigerstyle-check.sh
   ```

7. confirm the archived phase-1 result:
   - 6 warnings total
   - `aspen-time`: 2 `ambient_clock`
   - `aspen-core`: 4 `ambient_clock`
   - `aspen-hlc`: 0 starter findings

8. spotcheck the deferred package:

   ```bash
   scripts/tigerstyle-check.sh -p aspen-coordination -- --lib
   ```

9. confirm the archived defer decision:
   - 21 `ambient_clock`
   - 2 `compound_assertion`
   - 4 `no_panic`

Optional cleanup when finished:

```bash
git -C /path/to/aspen worktree remove --force "$sandbox/aspen"
rm -rf "$sandbox"
```

Archive references for that reconstruction:

- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-coordination-spotcheck.txt`
- `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
- `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`
