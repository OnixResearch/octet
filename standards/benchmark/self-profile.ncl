# Diagnostic compiler self-profile and Measureme attribution profile (#25).
#
# Export deterministic JSON with:
#   nickel export --format json standards/benchmark/self-profile.ncl \
#     > standards/benchmark/self-profile.json
let contracts = import "self-profile-contracts.ncl" in
{
  schema_version = "octet-compiler-self-profile/v1",
  profile_id = "rustc-self-profile-diagnostic",
  rustc_format_version = 9,
  self_profile_events = "default",
  configurations = [
    {
      config_id = "plain-compiler",
      description = "Plain cargo/rustc compilation without Octet driver or lint library.",
      claim_coverage_note = "Baseline compiler work only; provides no Tiger Style or architecture claims.",
      uses_octet_driver = false,
      enables_architecture_collection = false,
      enables_value_flow = false,
    },
    {
      config_id = "lint-collection",
      description = "Octet Dylint driver and lint library without architecture shard collection.",
      claim_coverage_note = "Enforces strict lints; provides no compiler architecture IR or value-flow claims.",
      uses_octet_driver = true,
      enables_architecture_collection = false,
      enables_value_flow = false,
    },
    {
      config_id = "architecture-collection",
      description = "Octet driver with compiler architecture IR collection, without selected value flow.",
      claim_coverage_note = "Emits compiler architecture shards and coverage without MIR value-flow provenance.",
      uses_octet_driver = true,
      enables_architecture_collection = true,
      enables_value_flow = false,
    },
    {
      config_id = "selected-value-flow",
      description = "Octet driver with compiler architecture IR and selected MIR value-flow provenance.",
      claim_coverage_note = "Full selected-MIR provenance and semantic body collection within declared budgets.",
      uses_octet_driver = true,
      enables_architecture_collection = true,
      enables_value_flow = true,
    },
  ],
  budgets = {
    max_mm_profdata_bytes = 67108864,
    max_events_per_unit = 500000,
    max_units = 64,
    phase_record_budget = 256,
  },
  non_claims = [
    "Instrumented -Z self-profile runs measure diagnostic compiler/collector behavior and remain separate from uninstrumented scored wall-time baselines.",
    "Summed rustc query or parallel thread spans are attribution counters, not elapsed wall-clock duration.",
    "A warm no-op Cargo run with NotCompiled units is a no-compilation observation, not evidence of zero-cost analysis.",
  ],
} | contracts.SelfProfileConfig
