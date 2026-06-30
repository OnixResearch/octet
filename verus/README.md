# Verus Proof Rails

Verus is an opt-in formal-proof evidence rail in Octet. It is not part of the
ordinary `cargo octet check` starter path, and it is not a Dylint lint.

The repository-owned fixture rail is intentionally small:

- source scope: `src/verified/addition.rs`
- proof target: `verus/proofs/addition_bound.rs`
- verifier surface: `nix run .#octet-verus -- --identity`
- proof command: `nix run .#octet-verus -- verus/proofs/addition_bound.rs`
- pass signal: `proof-result: pass octet-addition-bound`

`cargo octet prove` persists proof receipts under the selected artifact
directory and emits `formal_proof` provenance only after the receipt validates
its pass signal, source binding, verifier identity, trust-boundary lines, and
proof artifact identity.

Passing this rail is scoped evidence for the declared fixture property only. It
does not claim whole-program correctness, certification, business-domain
validation, semantic equivalence, or full safety.
