# Current verified Valence object evidence

Capture time: 2026-05-22T23:27:49Z

Pre-change committed baseline: `b00a06cf1039b1c54961c9fa8afd023bcae3880b` (`b00a06cf`)

This page indexes the focused evidence sweep for adding bounded MIR `branch-control-flow` selected-fixture evidence. It records command surfaces and observed pass signals; it does not embed generated receipts, move any readiness tag, or claim broader THIR or MIR support.

## Scope

The sweep covers selected local Octet/Valence review-evidence rails:

- Function Object v1 internal corpus replay and fixture freshness.
- Bounded fresh-target receipt determinism.
- Selected rustc HIR fixture spike replay/inventory coverage.
- THIR selected fixture inventory remains unchanged at `checked-arithmetic`, `call-return`, and `branch-control-flow`, with roundtrip coverage preserved.
- Selected rustc MIR fixture runtime and inventory coverage for exact `checked-arithmetic`, `call-return`, and `branch-control-flow` selectors.
- Named THIR inventory rail composed into the aggregate Valence gate.
- Named MIR inventory rail: `nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L`.
- Named object-corpus freshness rail: `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L`.
- Aggregate Valence object evidence composition, including the object-corpus freshness gate, THIR supplement inventory gate, MIR supplement fixture gate, and MIR supplement inventory gate.
- Canonical OpenSpec validation.

This is bounded local evidence replay. It is not a correctness proof, semantic-equivalence proof, exhaustive Rust coverage proof, pure-cache eligibility proof, public unattended production-readiness claim, cross-machine determinism proof, or cross-toolchain determinism proof.

## Repository state before implementation

```text
$ git status --short --branch
## main...origin/main
```

```text
$ git rev-parse HEAD
b00a06cf1039b1c54961c9fa8afd023bcae3880b
```

```text
$ openspec status
No active changes.
```

## Focused command sweep

Commands run from `/home/brittonr/git/octet`:

```bash
TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory -- --nocapture'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet cli::tests -- --nocapture'
TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_corpus_contract_metadata_reports_receipt_and_check_shapes -- --nocapture'
TMPDIR=/tmp nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L
TMPDIR=/tmp nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L
TMPDIR=/tmp nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L
```

Observed pass signals:

- `cargo fmt --check`: passed after formatting.
- `cargo test -p cargo-octet mir_fixture -- --nocapture`: passes; the focused MIR fixture tests cover the exact selected MIR fixture inventory (`checked-arithmetic`, `call-return`, `branch-control-flow`), inventory-emitted emit/check command roundtrips, positive branch-control-flow replay, default `unsupported-planned` boundaries, and non-proof boundary fields.
- `cargo test -p cargo-octet thir_inventory -- --nocapture`: passes; the focused THIR fixture tests preserve the exact selected THIR fixture inventory (`checked-arithmetic`, `call-return`, `branch-control-flow`) and its non-proof boundaries.
- `cargo test -p cargo-octet cli::tests -- --nocapture`: passes; CLI parsing accepts the explicit MIR `branch-control-flow` selected-fixture shape and still requires explicit MIR fixture selection.
- `cargo test -p cargo-octet object_corpus_contract_metadata_reports_receipt_and_check_shapes -- --nocapture`: passes; contract metadata names selected MIR fixture shapes without changing default/non-fixture `unsupported-planned` status.
- `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L`: passed; closeout rail for touched object-corpus inputs.
- `nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L`: passed; closeout rail for the updated MIR inventory.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`: passed; aggregate closeout rail including the updated MIR inventory and object-corpus freshness rail.

Benign environmental notes:

- Remote builder `[REDACTED]` failed to start on some Nix invocations; local Nix build proceeded and completed successfully.
- Compatibility alias crates warned that shared `src/main.rs` files appear in multiple binary targets; the focused tests still passed.

## Follow-up verification for this evidence page

Before committing this implementation slice, verify the docs/index and OpenSpec closeout with:

```bash
git diff --check
openspec validate --all --strict --json
```

and a focused read/search that confirms this page names the pre-change committed baseline, THIR `branch-control-flow` fixture coverage, MIR `branch-control-flow` fixture coverage, MIR inventory aggregate rail, object-corpus freshness rail, aggregate gate, pass signals, and non-claim boundaries.
