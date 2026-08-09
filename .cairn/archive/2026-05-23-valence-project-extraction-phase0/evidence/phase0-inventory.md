# Phase 0 Valence extraction inventory

This artifact records the current Octet-local Valence extraction surface for the `valence-project-extraction-phase0` Cairn package. It is an inventory and handoff checklist, not a code-movement proof.

## Repository snapshot

- Repository: `/home/brittonr/git/octet`
- Branch state before package creation: `main...origin/main`
- Active Cairn changes before package creation: none
- Latest landed Valence evidence release commit observed before this package: `8cee45e5 octet: add Valence evidence release gate`

## Candidate Valence-owned surfaces

These path families are candidate ownership for a future standalone Valence project because they implement or verify Rust object/evidence behavior:

- `cargo-octet/src/object.rs`
  - Function Object v1 emission/checking.
  - Object corpus receipt/check/caveat behavior.
  - Supplement emit/check/fixture behavior for selected HIR, THIR, and MIR families.
- `cargo-octet/src/evidence_index.rs`
  - Current-head Valence evidence page renderer/checker.
  - Required pass-signal and bounded non-claim checks.
- `cargo-octet/src/cli.rs`
  - `evidence-index` command wiring.
  - `object` command wiring and supplement family/scope arguments.
- `cargo-octet/tests/fixtures/object-corpus/**`
  - Internal corpus receipts.
  - Expanded corpus caveat/taxonomy fixtures.
  - Compatibility positive/negative fixtures.
- `cargo-octet/tests/fixtures/hir-spike/**`
  - Selected rustc HIR fixture spike inputs.
- `cargo-octet/tests/fixtures/thir-spike/**`
  - Selected rustc THIR fixture inputs for exact selectors currently named by evidence docs.
- `cargo-octet/tests/fixtures/mir-spike/**`
  - Selected rustc MIR fixture inputs for exact selectors currently named by evidence docs.
- `docs/current-head-valence-evidence.md`
  - Generated current-head human review page for bounded Valence evidence.
- `cairn/specs/valence-evidence/spec.md`
  - Accepted Valence evidence release/bundle contract.
- `cairn/archive/2026-05-23-valence-evidence-index-generator/**`
  - Archived evidence-index generator lifecycle package.
- `cairn/archive/2026-05-23-valence-evidence-release-gate/**`
  - Archived release evidence gate lifecycle package.

## Octet-retained surfaces for Phase 0

These remain Octet-owned unless a later Cairn package defines and proves a split:

- Root `octet` Dylint library in `src/**` and lint registration/config behavior.
- `tests/ui.rs`, `ui_tests/**`, and Dylint fixture/support rails.
- Tigerstyle/Slotcar compatibility wrappers and downstream rollout docs.
- `cargo-octet check`, `fingerprint`, `transcript`, `migrate`, and `roadmap` command families.
- `crates/slotcar-standards/**` unless later work decides Valence needs a separate standards adapter.

## Shared handoff surfaces

These are shared until extraction creates replacement locations or compatibility shims:

- `flake.nix`
  - Builds `cargo-octet` and current checks.
  - Exposes Valence evidence gates.
- Root `Cargo.toml`
  - Current workspace membership includes `cargo-octet` and `crates/slotcar-standards`.
- `docs/content-addressed-rust-evidence-boundaries.md`
  - Describes evidence boundaries that may need a new Valence home or cross-link.
- Cairn lifecycle history under `cairn/archive/**` for Valence-related packages.

## Handoff gates required before cutover

A future extraction/cutover must preserve direct, named equivalents for:

```bash
nix build .#checks.x86_64-linux.valence-evidence-release --no-link -L
nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L
nix run .#cairn -- validate --root .
```

It must also provide a compatibility answer or replacement for the current explicit binary evidence-index checker:

```bash
cargo run -p cargo-octet --bin cargo-octet -- evidence-index check --page docs/current-head-valence-evidence.md
```

## Bounded claims preserved

Phase 0 does not claim:

- correctness,
- semantic equivalence,
- exhaustive Rust coverage,
- general HIR/THIR/MIR extraction,
- public unattended production readiness,
- cross-machine determinism,
- cross-toolchain determinism.

The current evidence remains bounded local selected-corpus / selected-fixture replay evidence.
