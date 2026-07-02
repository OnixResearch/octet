let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails = {},
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules
