# Current verified Valence object evidence

Capture time: 2026-05-22T17:55:31Z

Verified implementation baseline: `4b3e0eebddfa43bc48b1a7cc1b4b32f6c885435d` (`4b3e0eeb`)

This page indexes the focused evidence sweep run after the MIR selected-fixture supplement contract landed. It records command surfaces and observed pass signals; it does not embed generated receipts or move any readiness tag. The sweep also exposed stale object-corpus fixture hashes from the MIR implementation boundary, so this evidence-index slice refreshes those checked-in fixture hashes before recording the green current-head sweep.

## Scope

The sweep covers selected local Octet/Valence review-evidence rails:

- Function Object v1 internal corpus replay and fixture freshness.
- Bounded fresh-target receipt determinism.
- Selected rustc HIR fixture spike replay/inventory coverage.
- Selected rustc THIR fixture inventory and roundtrip coverage for `checked-arithmetic` and `call-return`.
- Selected rustc MIR fixture runtime coverage for exact `checked-arithmetic`.
- Aggregate Valence object evidence composition, including the MIR supplement fixture gate.
- Canonical OpenSpec validation.

This is bounded local evidence replay. It is not a correctness proof, semantic-equivalence proof, exhaustive Rust coverage proof, pure-cache eligibility proof, public unattended production-readiness claim, cross-machine determinism proof, or cross-toolchain determinism proof.

## Repository state before sweep

```text
$ git status --short --branch
## main...origin/main
```

```text
$ openspec list
No active changes found.
```

```text
$ openspec validate --all --strict --json
summary.totals.items=14
summary.totals.passed=14
summary.totals.failed=0
standards-codification.issues=[INFO requirements[77] long requirement]
```

## Fixture refresh before final sweep

The first attempted current-head sweep failed in `object_internal_corpus` because the checked-in internal and expanded object-corpus receipt fixtures had stale `object_set_hash` values. Refreshing the fixture family changed only those hash fields:

```text
cargo-octet/tests/fixtures/object-corpus/internal-corpus-receipt.json
  object_set_hash: b3:b6584ba2a7dd620c9e9d6cd8379a26777fb4e964b9c931950b93a65226de43c4
cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json
  object_set_hash: b3:bf30173592e224bc12b676eded95e0c271af7a146d5355e3f67d1a55b9a8a456
```

A focused replay after refresh passed:

```text
cargo test -p cargo-octet object_internal_corpus -- --nocapture
12 passed; 0 failed
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

Observed pass signals:

- `cargo fmt --check`: passed.
- `cargo test -p cargo-octet object_internal_corpus -- --nocapture`: `12 passed; 0 failed`.
- Fresh-target object corpus receipt rail printed:
  - `fresh_target.receipt_runs=4`
  - `fresh_target.byte_stability: PASS`
  - `fresh_target.compared_fields_stable: PASS`
  - `fresh_target.boundary_claims_false: PASS`
  - `fresh_target.object_count=251`
  - `fresh_target.object_set_hash=b3:b6584ba2a7dd620c9e9d6cd8379a26777fb4e964b9c931950b93a65226de43c4`
- `cargo test -p cargo-octet thir_inventory -- --nocapture`: `2 passed; 0 failed`.
- `cargo test -p cargo-octet mir_fixture -- --nocapture`: `1 passed; 0 failed`.
- `cargo test -p cargo-octet rustc_hir_spike -- --nocapture`: `5 passed; 0 failed` for each alias binary target (`cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`).
- `cargo test -p cargo-octet hir_fixture_inventory_lists_closed_selectors_and_boundaries -- --nocapture`: `1 passed; 0 failed` for each alias binary target.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`: passed and printed:
  - `valence_object_evidence.object_corpus_gate: PASS`
  - `valence_object_evidence.compatibility_gate: PASS`
  - `valence_object_evidence.caveat_taxonomy_gate: PASS`
  - `valence_object_evidence.thir_supplement_inventory_gate: PASS`
  - `valence_object_evidence.mir_supplement_fixture_gate: PASS`
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

and a focused grep/read that confirms this page names the verified implementation baseline, MIR fixture gate, aggregate gate, pass signals, fixture refresh, and non-claim boundaries.
