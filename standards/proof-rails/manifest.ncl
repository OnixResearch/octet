let contracts = import "contracts.ncl" in
let default_timeout_ms = 30000 in
let default_log_max_bytes = 4096 in

{
  schema_version = "octet-proof-rail-manifest/v1",
  rails = {
    addition_bound = {
      backend = "verus",
      command = "octet-verus verus/proofs/addition_bound.rs",
      command_argv = ["octet-verus", "verus/proofs/addition_bound.rs"],
      source_paths = ["src/verified/addition.rs"],
      proof_paths = ["verus/proofs/addition_bound.rs"],
      receipt_paths = [
        "verus/proofs/addition_bound.receipt",
        "proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt",
      ],
      pass_signal = "proof-result: pass octet-addition-bound",
      trust_boundaries = ["fixture-scoped proof rail validates Octet proof plumbing and the declared addition-bound property only; it does not prove whole-program correctness"],
      receipt_binding = "blake3-command-and-source",
      receipt_provenance = {
        schema_version = "octet-verus-proof-receipt/v1",
        runner = "octet-standards",
        environment = "cargo-octet-prove",
        requires_backend = true,
        requires_trust_boundary = true,
      },
      receipt_verifier = {
        verifier_id = "octet-verus@fixture-v1",
        verifier_digest = "9f5a7f8c9e1d2c3b4a59687766554433221100ffeeddccbbaa99887766554433",
        verifier_binary_path = "octet-verus",
        verifier_binary_digest = "9f5a7f8c9e1d2c3b4a59687766554433221100ffeeddccbbaa99887766554433",
        toolchain_id = "verus-fixture-wrapper/v1",
        toolchain_digest = "9f5a7f8c9e1d2c3b4a59687766554433221100ffeeddccbbaa99887766554433",
      },
      verifier_mode = "fixture",
      checked_scope = "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0",
      timeout_ms = default_timeout_ms,
      stdout_max_bytes = default_log_max_bytes,
      stderr_max_bytes = default_log_max_bytes,
      environment_allowlist = ["PATH"],
    },
  },
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules
