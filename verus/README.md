# Verus Proof Rails

Verus is an opt-in formal-proof evidence rail in Octet. It is not part of the
ordinary `cargo octet check` starter path, and it is not a Dylint lint.

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
and emits `formal_proof` provenance only after the receipt validates its pass
signal, source binding, proof binding, verifier/solver/toolchain identity,
trust-boundary entries, checked scope, and proof artifact identity.

Passing the fixture rail is scoped evidence for the declared proof-plumbing
fixture only. Passing a production pilot is scoped evidence for that pilot's
declared property only: the addition-bound target or the reflexive-order target.
The negative fixture is not a proof; it exists only to show production Verus
failures fail closed. No rail claims whole-program correctness, certification,
business-domain validation, semantic equivalence, or full safety. Every
production rail must declare `verifier_mode = "production"` and pin the real
Verus verifier, solver, Rust toolchain, and Nix closure or closure-graph digest.
