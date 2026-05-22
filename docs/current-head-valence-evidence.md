# Current verified Valence object evidence

Capture time: 2026-05-22T15:50:49Z

Verified code commit: `925f1ebac23ad97cc5993ab220bb210b1927cc11` (`925f1eba`)

This page indexes the focused evidence sweep run for the verified code commit before the docs-only evidence-index commit. It records command surfaces and observed pass signals; it does not embed generated receipts or move any readiness tag.

## Scope

The sweep covers selected local Octet/Valence review-evidence rails:

- Function Object v1 internal corpus replay and fixture freshness.
- Bounded fresh-target receipt determinism.
- Selected rustc HIR fixture spike replay/inventory coverage.
- Selected rustc THIR fixture inventory and roundtrip coverage for `checked-arithmetic` and `call-return`.
- Aggregate Valence object evidence composition.
- Canonical OpenSpec validation.

This is bounded local evidence replay. It is not a correctness proof, semantic-equivalence proof, exhaustive Rust coverage proof, pure-cache eligibility proof, public unattended production-readiness claim, cross-machine determinism proof, or cross-toolchain determinism proof.

## Repository state before sweep

```text
$ git status --short --branch
## main...origin/main
```

```text
$ openspec validate --all --strict --json
summary.totals.items=14
summary.totals.passed=14
summary.totals.failed=0
standards-codification.issues=[]
```

No active OpenSpec changes were present.

## Focused command sweep

Command run from `/home/brittonr/git/octet`:

```bash
TMPDIR=/tmp nix develop -c bash -c 'set -euo pipefail
cargo fmt --check
cargo test -p cargo-octet object_internal_corpus -- --nocapture
cargo test -p cargo-octet thir_inventory -- --nocapture
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
  - `fresh_target.object_count=247`
  - `fresh_target.object_set_hash=b3:50744afa1d8298434e8a8ff9ed2814d22eb6c7df5850fd2c7238ad2b0198bba2`
- `cargo test -p cargo-octet thir_inventory -- --nocapture`: `2 passed; 0 failed`.
- `cargo test -p cargo-octet rustc_hir_spike -- --nocapture`: `5 passed; 0 failed` for each alias binary target (`cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`).
- `cargo test -p cargo-octet hir_fixture_inventory_lists_closed_selectors_and_boundaries -- --nocapture`: `1 passed; 0 failed` for each alias binary target.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`: passed and printed:
  - `valence_object_evidence.object_corpus_gate: PASS`
  - `valence_object_evidence.compatibility_gate: PASS`
  - `valence_object_evidence.caveat_taxonomy_gate: PASS`
  - `valence_object_evidence.thir_supplement_inventory_gate: PASS`
  - `valence_object_evidence.fresh_build_determinism_gate: PASS`
  - `valence_object_evidence.scope=bounded-local-selected-corpus-evidence-replay`
  - `valence_object_evidence.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism`

Benign environmental notes:

- Remote builder `ssh-ng://root@10.10.10.1` failed to start; local Nix build proceeded and completed successfully.
- Compatibility alias crates warned that shared `src/main.rs` files appear in multiple binary targets; the focused tests and aggregate gate still passed.

## Follow-up verification for this evidence page

After writing this page, verify the docs/index slice with:

```bash
git diff --check
openspec validate --all --strict --json
```

and a focused grep/read that confirms this page names the verified code commit, aggregate gate, pass signals, and non-claim boundaries.
