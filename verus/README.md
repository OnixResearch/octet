# Verus Proof Rails

Verus is an opt-in formal-proof evidence rail in Octet. It is not part of the
ordinary `cargo octet check` starter path, and it is not a Dylint lint.
The canonical review route lives in
[`docs/trust-boundary-evidence.md#verus-proof-path-lane`](../docs/trust-boundary-evidence.md#verus-proof-path-lane):
requirement or policy ID to Verus proof rail, checked Rust source scope, proof
target, verifier identity, structured receipt, `formal_proof` provenance, and
artifact verification. The broader checked-contract ladder is documented at
[`docs/trust-boundary-evidence.md#checked-contract-ladder`](../docs/trust-boundary-evidence.md#checked-contract-ladder), where Verus receipts occupy only the scoped formal-proof stage.

The repository-owned fixture rail is intentionally small:

- source scope: `src/verified/addition.rs`
- proof target: `verus/proofs/addition_bound.rs`
- verifier surface: `nix run .#octet-verus -- --identity`
- proof command argv: `octet-verus verus/proofs/addition_bound.rs`
- pass signal: `proof-result: pass octet-addition-bound`
- receipt path: `proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt`
- verifier mode: `fixture`

The production pilot rails run tiny proof targets with real Verus:

- verifier surface: `nix run .#octet-production-verus -- --identity`
- addition proof command argv: `octet-production-verus verus/proofs/addition_bound.rs`
- addition receipt path: `proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt`
- reflexive-order proof command argv: `octet-production-verus verus/proofs/ordering_reflexive.rs`
- reflexive-order receipt path: `proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt`
- pass signal for each production pilot: `verification results:: 1 verified, 0 errors`
- verifier mode for each production pilot: `production`
- pinned release: Verus `0.2026.04.03.21dfcd2`, Rust `1.94.0-x86_64-unknown-linux-gnu`, bundled Z3, and an Octet-generated Nix closure graph digest

The production negative fixture is failure-path evidence only:

- source selector: `verus/failure-fixtures/addition_bound_invalid_source.rs`
- invalid proof target: `verus/proofs/addition_bound_invalid.rs`
- proof command argv: `octet-production-verus verus/proofs/addition_bound_invalid.rs`
- expected result: real Verus exits non-zero, `cargo octet prove` reports `verifier-failure`, and no `formal_proof` provenance is emitted
- isolation: ordinary positive proof runs over `src/verified` do not select the invalid fixture

`cargo octet prove --list` shows selected rails without writing artifacts.
`cargo octet prove --explain` adds deterministic remediation details for stale
bindings, verifier mismatches, unmatched rails, and unbacked proof claims.

`cargo octet prove` executes configured argv only, captures bounded stdout/stderr
artifacts, writes structured JSON receipts under the selected artifact directory,
emits `proof-cards.json` as a human review index, and emits `formal_proof`
provenance only after the receipt validates its pass signal, source binding,
proof binding, verifier/solver/toolchain identity, trust-boundary entries,
checked scope, and proof artifact identity.

## Proof cards

`proof-cards.json` contains one generated card per selected rail. A card names
the proof or property ID, source requirement or policy ID, checked Rust source
scope, proof target path, verifier mode, verifier identity summary,
solver/toolchain summary, receipt path, artifact BLAKE3 identities, evidence
origin, evidence tier, status, trust-boundary notes, non-claims, and
`cargo octet artifact verify` command. The card is an index into
receipt/provenance evidence, not standalone evidence.

Current selected positive rails produce cards with these stable identities when
run through `cargo octet prove --source src/verified --artifact-dir artifacts/octet`:

| Proof/property ID | Mode | Source scope | Proof target | Receipt path |
| --- | --- | --- | --- | --- |
| `octet.verus_proof_rails.addition-bound#rail-0` | `fixture` | `src/verified/addition.rs` | `verus/proofs/addition_bound.rs` | `proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt` |
| `octet.verus_proof_rails.addition-bound-production#rail-0` | `production` | `src/verified/addition.rs` | `verus/proofs/addition_bound.rs` | `proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt` |
| `octet.verus_proof_rails.ordering-reflexive-production#rail-0` | `production` | `src/verified/ordering.rs` | `verus/proofs/ordering_reflexive.rs` | `proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt` |

A generated card may report `evidence_tier = formal_proof` only when it matches a
validated structured receipt and matching provenance record for the same source
scope, proof target, verifier mode, and artifact BLAKE3 identity. Imported
Trellis cards use `evidence_origin = external_trellis_import`, name the Trellis
repository revision plus source/proof/log/receipt BLAKE3 identities when present,
and stay `manual_review_required` unless Octet reruns the proof and validates an
Octet-owned receipt. BLAKE3 identifies exact artifact bytes and does not by
itself prove proof correctness. If the receipt is missing, stale, failed, fixture
evidence is presented as production evidence, external evidence is presented as
Octet formal proof, or wording broadens the claim, card validation downgrades the
card to `manual_review_required` and records diagnostics. Ordinary
`cargo octet check` does not require proof cards, imported bundles, or Verus
artifacts.

Passing the fixture rail is scoped evidence for the declared proof-plumbing
fixture only. Passing a production pilot is scoped evidence for that pilot's
declared property only: the addition-bound target or the reflexive-order target.
The negative fixture is not a proof; it exists only to show production Verus
failures fail closed. No rail claims whole-program correctness, certification,
business-domain validation, semantic equivalence, or full safety. Every
production rail must declare `verifier_mode = "production"` and pin the real
Verus verifier, solver, Rust toolchain, and Nix closure or closure-graph digest.
