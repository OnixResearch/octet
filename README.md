# Octet

Octet is a Rust-compatible toolchain and lint distribution for systems code that
should be stable through closed interfaces. It applies
[Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
rules: explicit shell/core boundaries, bounded resource use, positive naming,
compact functions, and reviewable suppressions.

The chemistry metaphor is intentional: an octet is a stable full valence shell.
Octet treats Rust-compatible code as inert by default across APIs, effects,
capabilities, proofs, and review surfaces; interaction happens only through
complete, explicit interfaces.

Octet currently ships 38 Dylint lints across 6 domains. The repo also owns a
separate standards registry/checker seam for policy traceability and
non-Dylint source-shape rules.

## Quick start

1. Copy `templates/dylint.toml` into the consumer workspace as `dylint.toml`.
2. Add runner metadata to the workspace `Cargo.toml`:

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

3. Run Octet through the first-class runner.

Installed cargo subcommand:

```bash
cargo octet check
```

Nix, from the canonical repo flake without installing into the consumer toolchain:

```bash
nix run github:OnixResearch/octet#cargo-octet -- check
```

Local checkout alternative:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Structured output and reviewable sidecars are first-class too:

```bash
cargo octet check --output-format sarif --artifact-dir artifacts/octet
```

For a strict local gate, copy `templates/pre-commit-config.yaml` to a consumer
repo and pin this repository as a pre-commit hook source. The `octet-deny-all`
hook runs the same runner with every Octet/Tiger Style lint promoted to `deny`.

Exit codes stay stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration or cargo-check failures

Outside Nix, install the cargo subcommand once, then follow
`docs/consumer-guide.md` for the repo-pinned `OCTET_DYLINT_REV` bootstrap
before running `cargo octet check`:

```bash
cargo install --path /path/to/octet/cargo-octet
```

4. Tune thresholds, disabled lints, and per-lint levels in `dylint.toml`.

Standards registry checks live behind a separate command surface:

```bash
nix run .#octet-policy-validate
nix run .#octet-standards -- --help
```

Start with `docs/consumer-guide.md` for the full consumer setup and rollout
path. Legacy `cargo slotcar` / `cargo tigerstyle` command names remain
compatibility aliases only.

## Release/operator smoke path

For release readiness, operators should be able to find the critical surfaces from
this README plus command help, without reading archived OpenSpec changes:

- `cargo octet check --cache --artifact-dir artifacts/octet` records
  effect-summary output in `summary.txt`, sidecar findings, and lint-cache
  `cache_status` fields.
- `cargo octet fingerprint --check --review-manifest artifacts/fingerprint-review.json src/lib.rs`
  verifies semantic identity fingerprints with BLAKE3-backed review records.
- `cargo octet migrate start --manifest artifacts/octet-migration.json`
  writes a deterministic migration manifest before safe application.
- `cargo octet transcript docs/examples/release-operator-smoke/transcript.md`
  validates executable transcript command blocks and expected-output assertions.

One bounded smoke artifact covers those surfaces together:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

The smoke transcript lives at
`docs/examples/release-operator-smoke/transcript.md`.

## Naming map

Octet is the canonical current identity. `Tiger Style` remains the source
philosophy. `slotcar` and `tigerstyle` names are legacy compatibility surfaces
until the migration tasks below retire or reclassify them; they are not the
preferred product identity for new docs, commands, config, or generated output.

| Surface | Canonical Octet name | Legacy / compatibility classification |
| --- | --- | --- |
| public site | `octet.dev` | `slotcar.dev` is historical until a site decision lands |
| project name | `Octet` / `octet` | `slotcar` is legacy product identity |
| GitHub repo reference | `github:OnixResearch/octet` | legacy repo slugs are obsolete |
| crate and lint package | `octet` | `slotcar`/`tigerstyle` package aliases are compatibility only |
| lint runner binary | `cargo-octet` | `cargo-slotcar` and `cargo-tigerstyle` route to the same engine as aliases |
| lint runner command | `cargo octet check` | `cargo slotcar check` and `cargo tigerstyle check` are transition aliases |
| standards checker binary | `octet-standards` | `slotcar-standards` / `tigerstyle-standards` are compatibility aliases |
| standards policy commands | `octet-policy-export`, `octet-policy-validate` | old policy command names are compatibility aliases |
| lint attrs and domain groups | `octet::<lint>` and `octet::<domain>` | `slotcar::<...>` remains source compatibility during migration |
| workspace runner config | `[workspace.metadata.octet]` | `[workspace.metadata.slotcar]` is legacy and lower precedence |
| lint tuning config | `[octet]` and `[octet.lint_levels]` | `[slotcar]` is legacy and lower precedence |
| support env prefix | `OCTET_*` | `TIGERSTYLE_*` is bootstrap compatibility while Dylint wiring migrates |

When both Octet and legacy config are present, Octet names win. Remaining
`Tiger Style` references are valid only when they describe the coding philosophy,
not the product/tool identity.

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
- `docs/consumer-guide.md` — first run, config split, Nix integration, pre-commit hook, output modes, artifact bundles, migration notes
- `templates/pre-commit-config.yaml` — copyable deny-all pre-commit hook configuration
- `docs/lint-reference.md` — full lint catalog, default levels, suppression paths, config reference
- `docs/contributor-guide.md` — source-of-truth map, doc-update crosswalk, verification rails
- `docs/downstream-rollout.md` — rollout profiles, promotion playbooks, and boundary guidance
- `docs/downstream-rollout-evidence.md` — Aspen-backed rollout evidence and historical replay paths
- `standards/policies/README.md` — typed policy registry and traceability contract
- `docs/standards/policy-contract.md` — standards rules mapped to verification rails
- `docs/examples/release-operator-smoke/transcript.md` — one bounded release smoke transcript covering operator surfaces
- `docs/release-candidate.md` — pre-tag release candidate checklist and local smoke receipt path
- `CHANGELOG.md` — breaking changes and notable updates for consumers

## License

AGPL-3.0-or-later
