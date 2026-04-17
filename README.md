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

3. Run tigerstyle through the first-class runner:

```bash
cargo tigerstyle check
nix run path:/path/to/tigerstyle#cargo-tigerstyle -- check
```

4. Tune thresholds, disabled lints, and per-lint levels in `dylint.toml`.

Start with `docs/consumer-guide.md` for the full consumer setup and rollout
path.

## Naming map

| Surface | Canonical name |
| --- | --- |
| repo and lint package | `tigerstyle` |
| binary | `cargo-tigerstyle` |
| command | `cargo tigerstyle check` |
| lint attrs and domain groups | `tigerstyle::<lint>` and `tigerstyle::<domain>` |
| workspace runner config | `[workspace.metadata.tigerstyle]` |
| lint tuning config | `[tigerstyle]` and `[tigerstyle.lint_levels]` |
| support env prefix | `TIGERSTYLE_*` |

These names are the current public contract. This docs refresh keeps them in
place; rename work is out of scope.

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

- `docs/consumer-guide.md` — first run, config split, Nix integration, migration notes
- `docs/lint-reference.md` — full lint catalog, default levels, suppression paths, config reference
- `docs/contributor-guide.md` — source-of-truth map, doc-update crosswalk, verification rails
- `docs/downstream-rollout.md` — deep rollout playbook and Aspen-backed rollout evidence

## License

AGPL-3.0-or-later
