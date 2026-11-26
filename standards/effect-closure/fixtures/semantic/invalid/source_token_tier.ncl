let contracts = import "../../../semantic-contracts.ncl" in
{
  schema_version = "octet-semantic-effect-profile/v1",
  profile_id = "invalid-tier",
  claim_tier = "selected-source-token-effect-closure",
  compiler_stage = "rustc-mir-optimized",
  unwind_mode = "modeled-cleanup",
  supported_subset = {
    subset_id = "bounded-sequential-rust-mir/v1",
    included_constructs = ["pure-arithmetic"],
    excluded_constructs = ["ffi-extern"],
  },
  effect_vocabulary = ["alloc", "unknown"],
  allowed_effects = [],
  denied_effects = ["unknown"],
  budgets = {
    max_bodies = 32,
    max_blocks_per_body = 64,
    max_steps = 1024,
    max_rounds = 8,
    call_edge_limit = 256,
  },
  non_claims = ["Invalid claim tier must fail contract validation."],
} | contracts.SemanticEffectProfile
