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

`cargo octet prove --list` shows selected rails without writing artifacts.
`cargo octet prove --explain` adds deterministic remediation details for stale
bindings, verifier mismatches, unmatched rails, and unbacked proof claims.

`cargo octet prove` executes configured argv only, captures bounded stdout/stderr
artifacts, writes structured JSON receipts under the selected artifact directory,
and emits `formal_proof` provenance only after the receipt validates its pass
signal, source binding, proof binding, verifier/solver/toolchain identity,
trust-boundary entries, checked scope, and proof artifact identity.

Passing the fixture rail is scoped evidence for the declared proof-plumbing
fixture only. It does not claim production Verus proof, whole-program correctness,
certification, business-domain validation, semantic equivalence, or full safety.
A production rail must declare `verifier_mode = "production"` and pin the real
Verus verifier, solver, Rust toolchain, and Nix closure or closure-graph digest.
