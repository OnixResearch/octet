let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails.addition_bound.receipt_verifier.verifier_id = "",
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules
