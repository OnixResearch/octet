# Contributor guide

This page maps public documentation back to the source files that define the
real contract.

For the landing page and naming table, see `../README.md`.
For consumer onboarding, see `consumer-guide.md`.
For the shipped lint catalog, see `lint-reference.md`.

[Assertion review guide](assertion-review.md)
[Assertion checklist template](../templates/assertion-checklist.md)
[Safety-critical claims guide](safety-critical.md)
[Bounds and pacing review guide](bounds-and-pacing-review.md)
[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)
[Safety spec contract](../openspec/specs/safety/spec.md)
[Resilience review guide](resilience-review.md)
[Five-whys architecture template](../templates/five-whys-architecture.md)
[Performance review guide](performance-review.md)
[Performance sketch template](../templates/performance-sketch.md)
[Performance spec contract](../openspec/specs/performance/spec.md)
[Concurrency and distributed-state review guide](concurrency-state-review.md)
[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)
[Concurrency spec contract](../openspec/specs/concurrency/spec.md)
[System comprehension guide](system-comprehension.md)
[System walkthrough template](../templates/system-walkthrough.md)
[OpenSpec proposal template](../openspec/templates/proposal.md)
[OpenSpec delta-spec template](../openspec/templates/spec.md)
[OpenSpec design template](../openspec/templates/design.md)
[Tiger Style policy registry](../standards/policies/README.md)
[Tiger Style standards contract](standards/policy-contract.md)
[Dynamic evidence profiles](dynamic-evidence-profiles.md)
[Dynamic evidence profile template](../templates/dynamic-evidence-profile.md)
[Rust idiom guidance](rust-idiom-guidance.md)
[Functional API boundary guidance](functional-api-boundaries.md)
[Unison-inspired Rust patterns](unison-inspired-rust-patterns.md)
[Function fingerprint guide](function-fingerprints.md)

## Source-of-truth map

| Contract area | Source of truth |
| --- | --- |
| lint inventory and registration | `src/lib.rs` |
| diagnostic wording, guide sections, domain groups | `src/diagnostics.rs` |
| config schema and defaults | `src/config.rs` |
| cargo subcommand parsing and engine behavior | `cargo-octet/src/cli.rs`, `cargo-octet/src/config.rs`, `cargo-octet/src/engine.rs` |
| standards registry and checker core | `standards/policies/`, `crates/slotcar-standards/src/` |
| standards source material | `docs/standards/source/` |
| published Nix entry points and helpers | `flake.nix` |
| consumer-owned template | `templates/dylint.toml` |
| landing page and naming map | `README.md` |
| consumer onboarding docs | `docs/consumer-guide.md` |
| lint catalog and config reference | `docs/lint-reference.md` |
| deep rollout playbook | `docs/downstream-rollout.md` |
| OpenSpec documentation contract | `openspec/specs/documentation/spec.md` |
| OpenSpec consumer runner contract | `openspec/specs/consumer-integration/spec.md` |
| UI fixtures and catalog coverage | `ui_tests/`, `tests/ui.rs`, `tests/fixtures/ui_fixture_set.json`, `tests/fixtures/lint_emitting_fixtures.json` |

## If you add or change a lint

Check these source files first:

- `src/lib.rs`
- `src/diagnostics.rs`
- the lint implementation under `src/<domain>/`

Then update the matching public docs and fixtures:

- `docs/lint-reference.md` for name, default level, domain, and summary
- `README.md` if the total lint count or domain summary changes
- `docs/consumer-guide.md` if suppression or rollout guidance changes
- `docs/downstream-rollout.md` if starter or inventory guidance changes
- `tests/fixtures/ui_fixture_set.json`
- `tests/fixtures/lint_emitting_fixtures.json`
- the relevant OpenSpec spec (`openspec/specs/documentation/spec.md` and domain specs such as `openspec/specs/safety/spec.md`)

## If you add or change a config key

Check these source files first:

- `src/config.rs`
- any lint implementation that reads the key

Then update:

- `docs/lint-reference.md` config table
- `docs/consumer-guide.md` if onboarding or tuning guidance changes
- `templates/dylint.toml`
- `README.md` if the naming or quick-start surface changes
- the relevant OpenSpec spec (`openspec/specs/documentation/spec.md` and any affected domain spec)

Remember:

- `dylint.toml` keys stay bare (`no_unwrap`)
- source attrs and group attrs stay canonical (`slotcar::no_unwrap`, `slotcar::safety`)

## If you add or change standards policy

Check these source files first:

- `standards/policies/registry.ncl`
- `standards/policies/contracts.ncl`
- `crates/slotcar-standards/src/`
- `standards/check-fixtures/`
- `docs/standards/source/`

Then update:

- `standards/policies/README.md`
- `docs/standards/policy-contract.md`
- `README.md` if command surfaces change
- `flake.nix` checks/apps when verification changes

Keep the seam intact: Dylint lints stay in `src/`; policy registry and
source-shape checks stay in `crates/slotcar-standards` and `standards/`.

## If you add or change a published entry point

Check these source files first:

- `cargo-octet/src/cli.rs`
- `cargo-octet/src/engine.rs`
- `flake.nix`

Then update:

- `README.md` naming map and quick-start section
- `docs/consumer-guide.md`
- `docs/downstream-rollout.md` if rollout guidance changes
- `tests/fixtures/consumer-integration/`
- `tests/consumer_integration.rs`
- `openspec/specs/documentation/spec.md`
- `openspec/specs/consumer-integration/spec.md`

If the change touches `TIGERSTYLE_*` names, keep the naming table in
`README.md` as the canonical public list and make deeper docs link back to it
instead of publishing conflicting variants.

## Operational gotchas

These are hard-won lessons that affect daily development in this repo.

### Nix devshell is mandatory

Plain `cargo` outside `nix develop` can miss rustc-private libraries,
OpenSSL/pkg-config, and the pinned nightly toolchain. Always wrap Rust commands:

```bash
nix develop -c bash -c 'cargo check'
```

### Refreshing UI snapshots

When lint diagnostic wording or help text changes, `.stderr` snapshots in
`ui_tests/` will drift. Refresh them with:

```bash
nix develop -c bash -c 'BLESS=1 cargo test --test ui ui -- --ignored --nocapture'
```

### Target directory lock contention

The shared `~/.cargo-target` directory can block on unrelated repo builds. Use a
temporary target directory when this happens:

```bash
tmp_target="$(mktemp -d /tmp/slotcar-target.XXXXXX)"
trap 'rm -rf "$tmp_target"' EXIT
CARGO_TARGET_DIR="$tmp_target" nix develop -c bash -c 'cargo test --lib'
```

### Tool-scoped attrs in standalone fixtures

Standalone UI fixtures that use canonical `slotcar::` attrs need:

```rust
#![feature(register_tool)]
#![register_tool(slotcar)]
```

Without this, rustc rejects the attrs before the lint can run. Bare lint names
like `#[allow(no_unwrap)]` work without the prelude.

### Staging new files before Nix checks

Git-backed flake sources do not include untracked files. `git add` new fixture
files, workspace members, and standards fixture trees before running
`nix flake check` or `nix develop`.

### Lockfile regeneration after dependency changes

After adding or removing Cargo dependencies, run `cargo generate-lockfile`
before trusting `--locked` Nix builds.

## Verification rails

Run Rust commands inside the Nix devshell here.

Common checks:

```bash
nix develop -c bash -c 'cargo check'
nix develop -c bash -c 'cargo test --lib'
nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'
nix develop -c bash -c 'cargo test -p slotcar-standards'
nix run .#octet-policy-validate
```

Useful doc-facing spot checks:

```bash
nix develop -c bash -c 'cargo test --test consumer_integration downstream_rollout_docs_point_to_first_class_cli_path -- --nocapture'
rg -n "cargo octet check|cargo-octet|octet::|\[octet\]|\[workspace.metadata.octet\]" README.md docs/*.md
```

For fingerprint docs/examples, use one review command:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

Run it whenever `cargo octet fingerprint` behavior or checked
`docs/examples/function-fingerprint-*` transcripts change. This aggregate check
is part of `nix flake check` and is the CI/operator gate for executable
fingerprint docs, including downstream smoke and the docs grep regression. Use
focused flake checks or direct Cargo filters only after the aggregate gate
identifies a failing slice.

## OpenSpec touchpoints

When public docs or public runner surfaces change:

- update the active change under `openspec/changes/<change>/`
- start proposals from `openspec/templates/proposal.md`
- start delta specs from `openspec/templates/spec.md`
- start designs from `openspec/templates/design.md`
- keep `tasks.md` in the repo template shape from `openspec/templates/tasks.md`
- update `openspec/specs/documentation/spec.md` when the docs contract changes
- update `openspec/specs/consumer-integration/spec.md` when command, Nix helper, or metadata behavior changes

If a change finishes every task, sync the delta spec into `openspec/specs/` and
archive the change immediately.
