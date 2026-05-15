# slotcar

Custom [Dylint](https://github.com/trailofbits/dylint) lints for Rust that apply
[Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
rules: explicit shell/core boundaries, bounded resource use, positive naming,
compact functions, and reviewable suppressions.

`slotcar` currently ships 38 lints across 6 domains. The repo also owns a
separate standards registry/checker seam for policy traceability and
non-Dylint source-shape rules.

## Quick start

1. Copy `templates/dylint.toml` into the consumer workspace as `dylint.toml`.
2. Add runner metadata to the workspace `Cargo.toml`:

```toml
[workspace.metadata.slotcar]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

3. Run slotcar through the first-class runner.

Nix:

```bash
nix run path:/path/to/slotcar#cargo-slotcar -- check
```

Structured output and reviewable sidecars are first-class too:

```bash
cargo slotcar check --output-format sarif --artifact-dir artifacts/slotcar
```

Exit codes stay stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration or cargo-check failures

Outside Nix, install the cargo subcommand once, then follow
`docs/consumer-guide.md` for the repo-pinned `TIGERSTYLE_DYLINT_REV` bootstrap
before running `cargo slotcar check`:

```bash
cargo install --path /path/to/slotcar/cargo-slotcar
```

4. Tune thresholds, disabled lints, and per-lint levels in `dylint.toml`.

Standards registry checks live behind a separate command surface:

```bash
nix run .#slotcar-policy-validate
nix run .#slotcar-standards -- --help
```

Start with `docs/consumer-guide.md` for the full consumer setup and rollout
path.

## Naming map

| Surface | Canonical name |
| --- | --- |
| public site | `slotcar.dev` |
| project name | `slotcar` |
| GitHub repo slug | `slotcar-rs` |
| crate and lint package | `slotcar` |
| lint runner binary | `cargo-slotcar` |
| lint runner command | `cargo slotcar check` |
| standards checker binary | `slotcar-standards` |
| standards policy commands | `slotcar-policy-export`, `slotcar-policy-validate` |
| lint attrs and domain groups | `slotcar::<lint>` and `slotcar::<domain>` |
| workspace runner config | `[workspace.metadata.slotcar]` |
| lint tuning config | `[slotcar]` and `[slotcar.lint_levels]` |
| support env prefix | `TIGERSTYLE_*` |

## Lint domains at a glance

| Domain | Count | Focus |
| --- | ---: | --- |
| safety | 18 | panics, bounds, defaults, loops, queues, unsafe boundaries, explicit failure paths |
| naming | 5 | predicate names, acronym casing, units, currency |
| structure | 10 | function size, nesting, parameters, casts, public APIs |
| purity | 3 | wall-clock boundaries and verified-module purity |
| concurrency | 1 | lock ordering |

See `docs/lint-reference.md` for the full catalog, defaults, suppression rules,
and config keys.

## Documentation

- [Assertion review guide](docs/assertion-review.md) and [checklist template](templates/assertion-checklist.md)
- [Safety-critical claims guide](docs/safety-critical.md)
- [Bounds and pacing review guide](docs/bounds-and-pacing-review.md) and [checklist template](templates/bounds-and-pacing-checklist.md)
- [Resilience review guide](docs/resilience-review.md) and [five-whys template](templates/five-whys-architecture.md)
- [Performance review guide](docs/performance-review.md) and [sketch template](templates/performance-sketch.md)
- [Concurrency and distributed-state review guide](docs/concurrency-state-review.md) and [checklist template](templates/concurrency-state-checklist.md)
- [System comprehension guide](docs/system-comprehension.md) and [walkthrough template](templates/system-walkthrough.md)
- `docs/consumer-guide.md` — first run, config split, Nix integration, output modes, artifact bundles, migration notes
- `docs/lint-reference.md` — full lint catalog, default levels, suppression paths, config reference
- `docs/contributor-guide.md` — source-of-truth map, doc-update crosswalk, verification rails
- `docs/downstream-rollout.md` — rollout profiles, promotion playbooks, and boundary guidance
- `docs/downstream-rollout-evidence.md` — Aspen-backed rollout evidence and historical replay paths
- `standards/policies/README.md` — typed policy registry and traceability contract
- `docs/standards/policy-contract.md` — standards rules mapped to verification rails
- `CHANGELOG.md` — breaking changes and notable updates for consumers

## License

AGPL-3.0-or-later
