let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v2",
  profile_id = "deterministic-core",
  binding_profile = "implementation-bound-v1",
  function_address_baseline = "evidence/octet/function-addresses.json",
  module_roles = [{ path = "src/lib.rs", role = "core" }],
  call_summaries = [{
    call_site_id = "b3:0000000000000000000000000000000000000000000000000000000000000000",
    review_label = "missing implementation binding in v2",
    callee = "ws_dep::pure_fn",
    provider = {
      kind = "workspace",
      name = "ws_dep",
      version = "0.1.0",
      source = "workspace-envelope:b3:1111111111111111111111111111111111111111111111111111111111111111",
      target = "x86_64-unknown-linux-gnu",
      features = [],
    },
    effects = [],
    evidence = {
      kind = "cargo-lock-review-v1",
      locator = "Cargo.lock#ws_dep",
      blake3 = "b3:1111111111111111111111111111111111111111111111111111111111111111",
    },
  }],
  non_claims = ["invalid fixture"],
} | contracts.WorkspacePolicy
