# Consumer guide

Start here when onboarding Octet into a Rust workspace. Octet is the canonical
project/toolchain identity; Tiger Style names the coding philosophy, and
`slotcar` / `tigerstyle` command or config names are compatibility aliases only.
This page covers the supported runner surface, where config lives, and which
deeper docs to read next.

For the landing page and naming map, see `../README.md`.
For the full lint catalog, see `lint-reference.md`.
For rollout profiles, see `downstream-rollout.md`.

## Supported entry points

Use the first-class Octet runner surface. Pick one of these paths:

Nix one-off:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Outside Nix, install the subcommand once, then export the bootstrap revision
used by this repo before running it:

```bash
cargo install --path /path/to/octet/cargo-octet
export OCTET_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9
cargo octet check
```

For repo-local one-offs without installation, reuse the same
`OCTET_DYLINT_REV` export and run the binary from the checkout:

```bash
cargo run --manifest-path /path/to/octet/cargo-octet/Cargo.toml -- check
```

For Nix consumers wiring Octet into `nix flake check`, use the first-party
helper exposed from `flake.nix`:

- `lib.mkConsumerCheck`

Historical `TIGERSTYLE_PATH` wrappers still exist in archived rollout evidence,
but they are replay-only. New consumers should start from one of the first-class
paths above plus `[workspace.metadata.octet]`.

## Quick start

### 1. Copy the consumer-owned lint config

Copy `templates/dylint.toml` into the workspace root as `dylint.toml`.

A minimal starting point can be as small as:

```toml
[octet]
```

Use the template comments when you need threshold overrides, allowlists,
`disabled_lints`, or `[octet.lint_levels]`. Legacy `[slotcar]` config remains
accepted only as a lower-precedence migration alias.

### 2. Add runner policy to the workspace `Cargo.toml`

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

Keep the split clear:

- `[workspace.metadata.octet]` owns runner policy
- `[octet]` in `dylint.toml` owns lint tuning
- legacy `[workspace.metadata.slotcar]` and `[slotcar]` are compatibility aliases; Octet config wins when both are present

Runner keys:

| Key | Meaning |
| --- | --- |
| `default_scope` | default `cargo check` scope when the command gets no explicit `-p` / `--workspace` flags |
| `cargo_check_args` | default cargo-check tail such as `--lib` |

### 3. Run the check

Run one of the supported entry points above from the workspace root. If you
installed `cargo-octet`, the common shape is:

```bash
cargo octet check
```

You can override scope or cargo-check args per run:

```bash
cargo octet check -p crate-a
cargo octet check -p crate-a -- --all-targets
```

You can also choose machine-readable output and a saved artifact bundle:

```bash
cargo octet check --output-format json
cargo octet check --output-format sarif --artifact-dir artifacts/octet
```

Once a rollout has known debt, capture it as an explicit acceptance baseline:

```bash
cargo octet check --output-format json --write-baseline octet-baseline.json
cargo octet check --output-format json --baseline octet-baseline.json
```

The baseline file records the current semantic/stable finding IDs plus enough
finding context for review. Semantic IDs are derived from the lint, crate,
message/suggestion, normalized snippet hash when available, config hash, and
Octet tool version; file paths remain display metadata so simple renames do not
create fresh debt. Future `--baseline` runs compare the current finding set to
that accepted ID set. Fully accepted warning-only findings keep exit `0`; any new
unaccepted stable ID fails the run with exit `1` and records `baseline` counts in
`status.json` / SARIF properties. Stale accepted IDs are reported so teams can
prune resolved debt without blocking the gate.

Exit codes are stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration/configuration failures, including underlying `cargo check` failures

When `--artifact-dir` is set, the runner writes `command.txt`, `status.json`, `summary.txt`, and the selected structured-output mirror (`results.jsonl` or `results.sarif`). JSON findings carry `stable_id`, `semantic_id`, `identity_status`, `normalized_snippet_hash`, `effect_family`, and `cache_status`; `status.json`, SARIF run properties, baseline evaluations, and stderr summaries include replay metadata such as the toolchain plus config/profile hashes. Downstream tools can compare runs without scraping ambient stderr.

### 3a. Plan structured migrations

Use the migration manifest commands when rolling out a new profile or policy set:

```bash
cargo octet migrate start --profile starter --manifest .octet/migrations/starter.json
cargo octet migrate status --manifest .octet/migrations/starter.json
cargo octet migrate apply-safe --manifest .octet/migrations/starter.json
```

`start` writes a deterministic manifest with config and crate-graph hashes.
`status` reports accepted/fixed/deferred/manual/unresolved counts. `apply-safe`
only records safe-apply state and preserves manual-review buckets; code edits
still flow through the existing machine-applicable cargo-fix rails.

### 3b. Verify executable documentation transcripts

Markdown examples can carry fenced transcript blocks with `$ ` command lines and
`=> ` expected-output assertions:

```octet-transcript
$ cargo octet check --output-format json
=> {"version":1
```

Verify a document with:

```bash
cargo octet transcript docs/examples/my-transcript.md --format json
```

The transcript report is content-addressed with `transcript_hash` so docs can be
stored as deterministic regression evidence.

### 4. Tune rollout controls in `dylint.toml`

Keep config keys bare in `dylint.toml`:

```toml
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
numeric_units = "warn"
```

Source attrs and group attrs stay canonical in Rust code:

```rust
#[allow(octet::unchecked_narrowing)]
#![deny(octet::naming)]
```

## Nix integration

The flake publishes the same runner surface through `nix run`,
`lib.mkConsumerCheck`, and `lib.mkPolicyArtifactConsumerCheck`.

Use `nix run` for one-off checks:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Use `lib.mkConsumerCheck` when you want Octet inside downstream flake
checks.

Use `lib.mkPolicyArtifactConsumerCheck` when a downstream flake should consume
the generated policy artifact bundle rather than reading the Nickel registry at
check time:

```nix
checks.${system}.octet-standards = octet.lib.mkPolicyArtifactConsumerCheck {
  inherit system;
  src = self;
  family = "embedded";
  sourceSubdirs = [
    "crates/my-core"
    "crates/my-shell"
  ];
  expectedDiagnostics = 0;
  requiredPolicyIds = [ "octet.embedded.bounded-queues" ];
};
```

The helper exports and asserts the generated bundle files, copies the consumer
source into the check sandbox, runs `octet-standards check
--policy-artifacts-dir ...`, asserts the expected diagnostic count, and can
assert required policy IDs are present in the policy-evaluation metadata. For
`family = "embedded"`, the generated artifact drives the
`octet.embedded.bounded-queues` structural gate: forbidden unbounded
constructors fail, while approved bounded APIs or wrappers need explicit
capacity/backpressure witnesses. Legacy `slotcar.*` policy IDs are compatibility
aliases only when a specific policy contract says so.

Wrapped Nix outputs already provide the runtime environment expected by the
runner, including:

- `OCTET_DRIVER_PATH`
- `OCTET_LINT_LIB`
- `OCTET_TOOLCHAIN`

Replay or bootstrap helpers may also set `OCTET_DYLINT_REV`.

## Runtime environment contract

The public runner surface uses the `OCTET_*` namespace for support wiring. Most
consumers should prefer the wrapped Nix app or repo-managed automation instead
of exporting these variables by hand.

Current canonical names that appear in wrappers and support tooling:

- `OCTET_DRIVER_PATH`
- `OCTET_LINT_LIB`
- `OCTET_TOOLCHAIN`
- `OCTET_DYLINT_REV` for driver bootstrap and replay helpers

The previous `TIGERSTYLE_*` variables are compatibility aliases while Dylint
bootstrap wiring migrates. `TIGERSTYLE_PATH` belongs to historical wrapper-based
replay flows, not the current recommended onboarding path.

## Migration note: `todo!()` escapes moved to `no_todo`

If a workspace still uses justified `todo!()` stubs under `allow(no_panic)`,
`allow(octet::no_panic)`, or legacy `allow(slotcar::no_panic)`, migrate them to
`no_todo`:

- `#[allow(no_todo)]`
- `#[allow(octet::no_todo)]`
- `#[allow(slotcar::no_todo)]` as a source-compatibility alias during migration

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
