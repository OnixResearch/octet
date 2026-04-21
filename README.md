# tigerstyle

Custom [Dylint](https://github.com/trailofbits/dylint) lints for Rust that apply
[Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
rules: explicit shell/core boundaries, bounded resource use, positive naming,
compact functions, and reviewable suppressions.

`tigerstyle` currently ships 34 lints across 5 domains.

## Quick start

1. Copy `templates/dylint.toml` into the consumer workspace as `dylint.toml`.
2. Add runner metadata to the workspace `Cargo.toml`:

```toml
[workspace.metadata.tigerstyle]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

3. Run tigerstyle through the first-class runner.

Nix:

```bash
nix run path:/path/to/tigerstyle#cargo-tigerstyle -- check
```

Structured output and reviewable sidecars are first-class too:

```bash
cargo tigerstyle check --output-format sarif --artifact-dir artifacts/tigerstyle
```

Exit codes stay stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration or cargo-check failures

Outside Nix, install the cargo subcommand once, then follow
`docs/consumer-guide.md` for the repo-pinned `TIGERSTYLE_DYLINT_REV` bootstrap
before running `cargo tigerstyle check`:

```bash
cargo install --path /path/to/tigerstyle/cargo-tigerstyle
```

4. Tune thresholds, disabled lints, and per-lint levels in `dylint.toml`.

Start with `docs/consumer-guide.md` for the full consumer setup and rollout
path.

## Naming map

| Surface | Canonical name |
| --- | --- |
| public site | `tigerstyle.dev` |
| project name | `tigerstyle` |
| GitHub repo slug | `tigerstyle-rs` |
| crate and lint package | `tigerstyle` |
| binary | `cargo-tigerstyle` |
| command | `cargo tigerstyle check` |
| lint attrs and domain groups | `tigerstyle::<lint>` and `tigerstyle::<domain>` |
| workspace runner config | `[workspace.metadata.tigerstyle]` |
| lint tuning config | `[tigerstyle]` and `[tigerstyle.lint_levels]` |
| support env prefix | `TIGERSTYLE_*` |

## Lint domains at a glance

| Domain | Count | Focus |
| --- | ---: | --- |
| safety | 17 | panics, bounds, defaults, loops, queues, explicit failure paths |
| naming | 5 | predicate names, acronym casing, units, currency |
| structure | 8 | function size, nesting, parameters, casts, public APIs |
| purity | 3 | wall-clock boundaries and verified-module purity |
| concurrency | 1 | lock ordering |

See `docs/lint-reference.md` for the full catalog, defaults, suppression rules,
and config keys.

## Documentation

[Assertion review guide](docs/assertion-review.md)
[Assertion checklist template](templates/assertion-checklist.md)
[Safety-critical claims guide](docs/safety-critical.md)
[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)
[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)
[Resilience review guide](docs/resilience-review.md)
[Five-whys architecture template](templates/five-whys-architecture.md)
[Performance review guide](docs/performance-review.md)
[Performance sketch template](templates/performance-sketch.md)
[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)
[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)
[System comprehension guide](docs/system-comprehension.md)
[System walkthrough template](templates/system-walkthrough.md)
- `docs/consumer-guide.md` — first run, config split, Nix integration, output modes, artifact bundles, migration notes
- `docs/lint-reference.md` — full lint catalog, default levels, suppression paths, config reference
- `docs/contributor-guide.md` — source-of-truth map, doc-update crosswalk, verification rails
- `docs/downstream-rollout.md` — deep rollout playbook and Aspen-backed rollout evidence

## License

AGPL-3.0-or-later
