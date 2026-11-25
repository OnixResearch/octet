# Copy this file with standards/effect-closure/contracts.ncl, update the import,
# review every exact call site, then export with:
#   nickel export --format json effect-closure-policy.ncl > octet-effect-closure.json
let contracts = import "../standards/effect-closure/contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v2",
  profile_id = "deterministic-core",
  binding_profile = "implementation-bound-v1",
  function_address_baseline = "artifacts/octet-observe/function-addresses.json",
  module_roles = [
    { path = "crates/domain-core/src/lib.rs", role = "core" },
  ],
  call_summaries = [],
  non_claims = [
    "This workspace policy supports conservative selected source-token closure only; it does not establish complete Rust name resolution, MIR call-graph closure, or runtime reachability.",
    "Implementation-bound reviewed call summaries remain reviewed assumptions bound to current implementation bytes and build inputs; they are not formal proofs and do not establish whole-program correctness or release eligibility.",
  ],
} | contracts.WorkspacePolicy
