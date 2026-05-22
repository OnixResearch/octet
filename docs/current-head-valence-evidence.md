# Current verified Valence object evidence

Capture time: 2026-05-22T22:46:45Z

Verified implementation baseline: `c59534d3acd4b5b2cdf3c19d3818e6ac0c8f89fe` (`c59534d3`)

This page indexes the focused evidence sweep run after the THIR `branch-control-flow` selected-fixture evidence landed. It records command surfaces and observed pass signals; it does not embed generated receipts, move any readiness tag, or claim broader THIR or MIR support. This documentation/fixture-refresh slice also refreshes stale checked-in object-corpus receipt fixtures that the current-head sweep caught before the final pass.

## Scope

The sweep covers selected local Octet/Valence review-evidence rails:

- Function Object v1 internal corpus replay and fixture freshness.
- Bounded fresh-target receipt determinism.
- Selected rustc HIR fixture spike replay/inventory coverage.
- Selected rustc THIR fixture inventory and roundtrip coverage for `checked-arithmetic`, `call-return`, and `branch-control-flow`.
- Selected rustc MIR fixture runtime and inventory coverage for exact `checked-arithmetic` and `call-return` selectors; MIR `branch-control-flow` remains unsupported.
- Named THIR inventory rail composed into the aggregate Valence gate.
- Named MIR inventory rail: `nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L`.
- Aggregate Valence object evidence composition, including the THIR supplement inventory gate, MIR supplement fixture gate, and MIR supplement inventory gate.
- Canonical OpenSpec validation.

This is bounded local evidence replay. It is not a correctness proof, semantic-equivalence proof, exhaustive Rust coverage proof, pure-cache eligibility proof, public unattended production-readiness claim, cross-machine determinism proof, or cross-toolchain determinism proof.

## Repository state before refresh

```text
$ git status --short --branch
## main...origin/main
```

```text
$ git rev-parse HEAD
c59534d3acd4b5b2cdf3c19d3818e6ac0c8f89fe
```

```text
$ openspec status
No active changes.
```

## Focused command sweep

Command run from `/home/brittonr/git/octet`:

```bash
TMPDIR=/tmp nix develop -c bash -c 'set -euo pipefail
cargo fmt --check
cargo test -p cargo-octet object_internal_corpus -- --nocapture
cargo test -p cargo-octet thir_inventory -- --nocapture
cargo test -p cargo-octet mir_fixture -- --nocapture
cargo test -p cargo-octet rustc_hir_spike -- --nocapture
cargo test -p cargo-octet hir_fixture_inventory_lists_closed_selectors_and_boundaries -- --nocapture
nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L
'
```

The first current-head run caught stale checked-in object-corpus receipt fixtures. The refresh regenerated:

- `cargo-octet/tests/fixtures/object-corpus/internal-corpus-receipt.json`
- `cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json`

Observed pass signals after the fixture refresh:

- `cargo fmt --check`: passed.
- `cargo test -p cargo-octet object_internal_corpus -- --nocapture`: `12 passed; 0 failed`.
- Fresh-target object corpus receipt rail printed:
  - `fresh_target.receipt_runs=4`
  - `fresh_target.byte_stability: PASS`
  - `fresh_target.compared_fields_stable: PASS`
  - `fresh_target.boundary_claims_false: PASS`
  - `fresh_target.object_count=251`
  - `fresh_target.object_set_hash=b3:032eb6fe659751f643ca06edc31831c65d817db5c65dfbd95fdad5cb1efda5c7`
- Checked-in expanded object corpus receipt now records `object_count=487` and `object_set_hash=b3:def3ed7bde6a81f6914cac7fd781c3958c3cad8b99a088304826af6c167b85bf`.
- `cargo test -p cargo-octet thir_inventory -- --nocapture`: passes; the focused THIR fixture tests cover the exact selected THIR fixture inventory (`checked-arithmetic`, `call-return`, `branch-control-flow`), inventory-emitted emit/check command roundtrips, positive branch-control-flow replay, default `unsupported-planned` boundaries, and non-proof boundary fields.
- `cargo test -p cargo-octet mir_fixture -- --nocapture`: passes; the focused MIR fixture tests cover the exact selected MIR fixture inventory (`checked-arithmetic`, `call-return`), inventory-emitted emit/check command roundtrips, positive emit/check replay, unsupported selector rejection, and default `unsupported-planned` boundaries.
- `cargo test -p cargo-octet rustc_hir_spike -- --nocapture`: `5 passed; 0 failed` for each alias binary target (`cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`).
- `cargo test -p cargo-octet hir_fixture_inventory_lists_closed_selectors_and_boundaries -- --nocapture`: `1 passed; 0 failed` for each alias binary target.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`: passed and printed:
  - `valence_object_evidence.object_corpus_gate: PASS`
  - `valence_object_evidence.compatibility_gate: PASS`
  - `valence_object_evidence.caveat_taxonomy_gate: PASS`
  - `valence_object_evidence.thir_supplement_inventory_gate: PASS`
  - `valence_object_evidence.mir_supplement_fixture_gate: PASS`
  - `valence_object_evidence.mir_supplement_inventory_gate: PASS`
  - `valence_object_evidence.fresh_build_determinism_gate: PASS`
  - `valence_object_evidence.scope=bounded-local-selected-corpus-evidence-replay`
  - `valence_object_evidence.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism`

Benign environmental notes:

- Remote builder `[REDACTED]` failed to start on some Nix invocations; local Nix build proceeded and completed successfully.
- Compatibility alias crates warned that shared `src/main.rs` files appear in multiple binary targets; the focused tests and aggregate gate still passed.

## Follow-up verification for this evidence page

After writing this page, verify the docs/index slice with:

```bash
git diff --check
openspec validate --all --strict --json
```

and a focused read/search that confirms this page names the verified implementation baseline, THIR `branch-control-flow` fixture coverage, MIR `call-return` fixture coverage, MIR inventory aggregate rail, aggregate gate, pass signals, refreshed object-corpus receipt fixtures, and non-claim boundaries.
