let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v2",
  profile_id = "deterministic-core",
  binding_profile = "implementation-bound-v1",
  function_address_baseline = "evidence/octet/function-addresses.json",
  module_roles = [{ path = "src/lib.rs", role = "core" }],
  call_summaries = [{
    call_site_id = "b3:0000000000000000000000000000000000000000000000000000000000000000",
    review_label = "invalid justification value",
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
      kind = "reviewed-implementation-summary-v1",
      locator = "workspace-envelope#ws_dep",
      blake3 = "b3:1111111111111111111111111111111111111111111111111111111111111111",
    },
    implementation_binding = {
      profile = "implementation-bound-v1",
      package_envelope_blake3 = "b3:2222222222222222222222222222222222222222222222222222222222222222",
      closure_blake3 = "b3:1111111111111111111111111111111111111111111111111111111111111111",
      build_context_blake3 = "b3:3333333333333333333333333333333333333333333333333333333333333333",
      justification = "ambient-guess",
    },
  }],
  non_claims = ["invalid fixture"],
} | contracts.WorkspacePolicy
