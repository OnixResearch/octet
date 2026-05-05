# Consumer guide

Start here when onboarding `tigerstyle` into a Rust workspace. This page covers
the supported runner surface, where config lives, and which deeper docs to read
next.

For the landing page and naming map, see `../README.md`.
For the full lint catalog, see `lint-reference.md`.
For rollout profiles, see `downstream-rollout.md`.

## Supported entry points

Use the first-class runner surface. Pick one of these paths:

Nix one-off:

```bash
nix run path:/path/to/tigerstyle#cargo-tigerstyle -- check
```

Outside Nix, install the subcommand once, then export the bootstrap revision
used by this repo before running it:

```bash
cargo install --path /path/to/tigerstyle/cargo-tigerstyle
export TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9
cargo tigerstyle check
```

For repo-local one-offs without installation, reuse the same
`TIGERSTYLE_DYLINT_REV` export and run the binary from the checkout:

```bash
cargo run --manifest-path /path/to/tigerstyle/cargo-tigerstyle/Cargo.toml -- check
```

For Nix consumers wiring tigerstyle into `nix flake check`, use the first-party
helper exposed from `flake.nix`:

- `lib.mkConsumerCheck`

Historical `TIGERSTYLE_PATH` wrappers still exist in archived rollout evidence,
but they are replay-only. New consumers should start from one of the first-class
paths above plus `[workspace.metadata.tigerstyle]`.

## Quick start

### 1. Copy the consumer-owned lint config

Copy `templates/dylint.toml` into the workspace root as `dylint.toml`.

A minimal starting point can be as small as:

```toml
[tigerstyle]
```

Use the template comments when you need threshold overrides, allowlists,
`disabled_lints`, or `[tigerstyle.lint_levels]`.

### 2. Add runner policy to the workspace `Cargo.toml`

```toml
[workspace.metadata.tigerstyle]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

Keep the split clear:

- `[workspace.metadata.tigerstyle]` owns runner policy
- `[tigerstyle]` in `dylint.toml` owns lint tuning

Runner keys:

| Key | Meaning |
| --- | --- |
| `default_scope` | default `cargo check` scope when the command gets no explicit `-p` / `--workspace` flags |
| `cargo_check_args` | default cargo-check tail such as `--lib` |

### 3. Run the check

Run one of the supported entry points above from the workspace root. If you
installed `cargo-tigerstyle`, the common shape is:

```bash
cargo tigerstyle check
```

You can override scope or cargo-check args per run:

```bash
cargo tigerstyle check -p crate-a
cargo tigerstyle check -p crate-a -- --all-targets
```

You can also choose machine-readable output and a saved artifact bundle:

```bash
cargo tigerstyle check --output-format json
cargo tigerstyle check --output-format sarif --artifact-dir artifacts/tigerstyle
```

Once a rollout has known debt, capture it as an explicit acceptance baseline:

```bash
cargo tigerstyle check --output-format json --write-baseline tigerstyle-baseline.json
cargo tigerstyle check --output-format json --baseline tigerstyle-baseline.json
```

The baseline file records the current stable finding IDs plus enough finding
context for review. Future `--baseline` runs compare the current finding set to
that accepted ID set. Fully accepted warning-only findings keep exit `0`; any new
unaccepted stable ID fails the run with exit `1` and records `baseline` counts in
`status.json` / SARIF properties. Stale accepted IDs are reported so teams can
prune resolved debt without blocking the gate.

Exit codes are stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration/configuration failures, including underlying `cargo check` failures

When `--artifact-dir` is set, the runner writes `command.txt`, `status.json`, `summary.txt`, and the selected structured-output mirror (`results.jsonl` or `results.sarif`). JSON findings carry a stable `stable_id`, and `status.json`, SARIF run properties, baseline evaluations, and stderr summaries include replay metadata such as the toolchain plus config/profile hashes. Downstream tools can compare runs without scraping ambient stderr.

### 4. Tune rollout controls in `dylint.toml`

Keep config keys bare in `dylint.toml`:

```toml
[tigerstyle]
disabled_lints = ["no_unwrap"]

[tigerstyle.lint_levels]
numeric_units = "warn"
```

Source attrs and group attrs stay canonical in Rust code:

```rust
#[allow(tigerstyle::unchecked_narrowing)]
#![deny(tigerstyle::naming)]
```

## Nix integration

The flake publishes the same runner surface through `nix run`,
`lib.mkConsumerCheck`, and `lib.mkPolicyArtifactConsumerCheck`.

Use `nix run` for one-off checks:

```bash
nix run path:/path/to/tigerstyle#cargo-tigerstyle -- check
```

Use `lib.mkConsumerCheck` when you want tigerstyle inside downstream flake
checks.

Use `lib.mkPolicyArtifactConsumerCheck` when a downstream flake should consume
the generated policy artifact bundle rather than reading the Nickel registry at
check time:

```nix
checks.${system}.tigerstyle-standards = tigerstyle.lib.mkPolicyArtifactConsumerCheck {
  inherit system;
  src = self;
  family = "embedded";
  sourceSubdirs = [
    "crates/my-core"
    "crates/my-shell"
  ];
  expectedDiagnostics = 0;
  requiredPolicyIds = [ "tigerstyle.embedded.bounded-queues" ];
};
```

The helper exports and asserts the generated bundle files, copies the consumer
source into the check sandbox, runs `tigerstyle-standards check
--policy-artifacts-dir ...`, asserts the expected diagnostic count, and can
assert required policy IDs are present in the policy-evaluation metadata.

Wrapped Nix outputs already provide the runtime environment expected by the
runner, including:

- `TIGERSTYLE_DRIVER_PATH`
- `TIGERSTYLE_LINT_LIB`
- `TIGERSTYLE_TOOLCHAIN`

Replay or bootstrap helpers may also set `TIGERSTYLE_DYLINT_REV`.

## Runtime environment contract

The public runner surface uses the `TIGERSTYLE_*` namespace for support wiring.
Most consumers should prefer the wrapped Nix app or repo-managed automation
instead of exporting these variables by hand.

Current names that appear in wrappers and support tooling:

- `TIGERSTYLE_DRIVER_PATH`
- `TIGERSTYLE_LINT_LIB`
- `TIGERSTYLE_TOOLCHAIN`
- `TIGERSTYLE_DYLINT_REV` for driver bootstrap and replay helpers

Outside Nix, `TIGERSTYLE_DYLINT_REV` is the simplest bootstrap knob when your
wrapper does not already provide `TIGERSTYLE_DRIVER_PATH`.

`TIGERSTYLE_PATH` belongs to historical wrapper-based replay flows, not the
current recommended onboarding path.

## Migration note: `todo!()` escapes moved to `no_todo`

If a workspace still uses justified `todo!()` stubs under `allow(no_panic)` or
`allow(tigerstyle::no_panic)`, migrate them to `no_todo`:

- `#[allow(no_todo)]`
- `#[allow(tigerstyle::no_todo)]`

`no_panic` now covers only:

- `panic!()`
- `unimplemented!()`
- `unreachable!()`

Use `unjustified_no_todo_allow` to keep stub escapes documented and local.

## Next docs

- `lint-reference.md` for the full lint catalog, defaults, and config keys
- `downstream-rollout.md` for smoke, boundary-first, and inventory-only rollout profiles
- `downstream-rollout-evidence.md` for Aspen-backed evidence and historical replay paths
- `contributor-guide.md` if you are changing lints, config keys, or published runner surfaces
