# Current verified Valence object evidence

Capture time: 2026-05-23T01:44:48Z

Verified code commit: `c7f01d550f292818434aa81b3cbd73c65e906f11` (`c7f01d55`)

This page indexes the focused evidence sweep after landing bounded supplement receipt compatibility / fail-closed replay coverage. It records command surfaces and observed pass signals; it does not embed generated receipts, move any readiness tag, or claim broader HIR, THIR, or MIR extraction support.

## Scope

The sweep covers selected local Octet/Valence review-evidence rails:

- Function Object v1 internal corpus replay and fixture freshness.
- Object corpus compatibility fixtures and caveat taxonomy replay.
- Bounded fresh-target and fresh-build object corpus determinism.
- Selected rustc HIR fixture spike replay/inventory coverage.
- THIR selected fixture inventory for exact `checked-arithmetic`, `call-return`, and `branch-control-flow` selectors.
- MIR selected fixture runtime/inventory coverage for exact `checked-arithmetic`, `call-return`, and `branch-control-flow` selectors.
- Supplement receipt compatibility/fail-closed replay for positive v1 receipts, malformed JSON, missing replay-critical fields, unsupported future schema versions, wrong receipt families, and boundary tampering.
- Named supplement compatibility rail: `nix build .#checks.x86_64-linux.supplement-compatibility-gate --no-link -L`.
- Aggregate Valence object evidence composition, including object-corpus replay, object-corpus compatibility, caveat taxonomy, object-corpus freshness, supplement compatibility, THIR supplement inventory, MIR supplement fixture, MIR supplement inventory, and fresh-build determinism rails.
- Canonical OpenSpec validation.

This is bounded local selected-corpus / selected-fixture evidence replay. It is not a correctness proof, semantic-equivalence proof, exhaustive Rust coverage proof, pure-cache eligibility proof, public unattended production-readiness claim, cross-machine determinism proof, cross-toolchain determinism proof, or general HIR/THIR/MIR extraction claim.

## Repository state before docs refresh

```text
$ git status --short --branch
## main...origin/main
```

```text
$ git rev-parse HEAD
c7f01d550f292818434aa81b3cbd73c65e906f11
```

```text
$ openspec list
No active changes found.
```

## Focused command sweep

Commands run from `/home/brittonr/git/octet`:

```bash
TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_internal_corpus -- --nocapture'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory -- --nocapture'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet supplement_compatibility -- --nocapture'
TMPDIR=/tmp nix build .#checks.x86_64-linux.supplement-compatibility-gate --no-link -L
TMPDIR=/tmp nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L
```

Observed pass signals:

- `cargo fmt --check`: passed.
- `cargo test -p cargo-octet object_internal_corpus -- --nocapture`: passed; the selected internal corpus replay, fixture freshness, caveat taxonomy, compatibility, and fresh-target object-corpus checks remain green.
- `cargo test -p cargo-octet thir_inventory -- --nocapture`: passed; the focused THIR fixture tests preserve exact selected THIR fixture inventory (`checked-arithmetic`, `call-return`, `branch-control-flow`) and non-proof boundaries.
- `cargo test -p cargo-octet mir_fixture -- --nocapture`: passed; the focused MIR fixture tests preserve exact selected MIR fixture inventory (`checked-arithmetic`, `call-return`, `branch-control-flow`), inventory-emitted emit/check command roundtrips, and default `unsupported-planned` boundaries.
- `cargo test -p cargo-octet supplement_compatibility -- --nocapture`: passed; the focused compatibility test accepts a positive v1 supplement receipt and fails closed for malformed, missing-field, future-version, wrong-family, and boundary-tamper inputs.
- `nix build .#checks.x86_64-linux.supplement-compatibility-gate --no-link -L`: passed; the named rail runs the supplement compatibility replay test under Nix.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`: passed; aggregate output includes `valence_object_evidence.supplement_compatibility_gate: PASS` along with the existing object corpus, THIR, MIR, and fresh-build rails.

Aggregate pass lines expected from the current composition:

```text
valence_object_evidence.object_corpus_gate: PASS
valence_object_evidence.compatibility_gate: PASS
valence_object_evidence.caveat_taxonomy_gate: PASS
valence_object_evidence.object_corpus_fixture_freshness_gate: PASS
valence_object_evidence.supplement_compatibility_gate: PASS
valence_object_evidence.thir_supplement_inventory_gate: PASS
valence_object_evidence.mir_supplement_fixture_gate: PASS
valence_object_evidence.mir_supplement_inventory_gate: PASS
valence_object_evidence.fresh_build_determinism_gate: PASS
valence_object_evidence.scope=bounded-local-selected-corpus-evidence-replay
valence_object_evidence.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism
```

Benign environmental notes:

- Remote builder `[REDACTED]` may fail to start on some Nix invocations; local Nix build can proceed and complete successfully.
- Compatibility alias crates may warn that shared `src/main.rs` files appear in multiple binary targets; the focused tests still pass.

## Follow-up verification for this evidence page

Before committing this docs refresh, verify the docs/index and OpenSpec closeout with:

```bash
git diff --check
openspec validate --all --strict --json
```

and a focused read/search that confirms this page names the verified code commit, supplement compatibility rail, aggregate PASS signal, selected THIR/MIR fixture boundaries, pass signals, and non-claim boundaries.
