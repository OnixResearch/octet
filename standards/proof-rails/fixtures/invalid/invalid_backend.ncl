let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails.addition_bound.backend = "coq",
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules
