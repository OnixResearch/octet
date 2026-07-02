let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails.consumer_range_proof = manifest.rails.addition_bound & {
    command = "octet-verus verus/proofs/consumer_range.rs",
    command_argv = ["octet-verus", "verus/proofs/consumer_range.rs"],
    source_paths = ["src/verified/consumer_range.rs"],
    proof_paths = ["verus/proofs/consumer_range.rs"],
    receipt_paths = ["proofs/consumer-range-rail-0.receipt"],
    pass_signal = "proof-result: pass consumer-range-proof",
    trust_boundaries = ["consumer-scoped Verus rail proves only the declared range property; it does not prove whole-program correctness"],
    checked_scope = "formal-proof:consumer:consumer.range-proof#rail-0",
  },
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules
