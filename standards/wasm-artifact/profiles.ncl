let contracts = import "contracts.ncl" in
let maximum_artifact_bytes = 1048576 in
let maximum_imports = 32 in
let maximum_exports = 64 in
let maximum_memories = 1 in
let maximum_tables = 1 in
let maximum_functions = 128 in
{
  schema_version = "octet-wasm-artifact-profiles/v1",
  cohort = {
    cohort_id = "wasm-tools-0.253.0",
    identity = "b3:c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a",
    members = [
      { crate_name = "wasm-encoder", version = "0.253.0", role = "fixture-encoding" },
      { crate_name = "wasm-metadata", version = "0.253.0", role = "metadata-inspection" },
      { crate_name = "wasm-mutate", version = "0.253.0", role = "bounded-mutation" },
      { crate_name = "wasm-shrink", version = "0.253.0", role = "bounded-shrinking" },
      { crate_name = "wasm-smith", version = "0.253.0", role = "bounded-generation" },
      { crate_name = "wasmparser", version = "0.253.0", role = "validation-and-facts" },
      { crate_name = "wit-component", version = "0.253.0", role = "component-metadata" },
      { crate_name = "wit-parser", version = "0.253.0", role = "wit-interface-facts" },
      { crate_name = "wit-smith", version = "0.253.0", role = "bounded-wit-generation" },
    ],
  },
  profiles = [
    {
      profile_id = "portable-component-baseline",
      summary = "Execution-free static facts for portable core modules and components with no ambient host imports.",
      allowed_artifact_kinds = ["module", "component"],
      allowed_proposals = ["wasm1", "component-model"],
      allowed_worlds = [],
      allowed_imports = [],
      allow_any_exports = true,
      resource_limits = {
        max_artifact_bytes = maximum_artifact_bytes,
        max_imports = maximum_imports,
        max_exports = maximum_exports,
        max_memories = maximum_memories,
        max_tables = maximum_tables,
        max_functions = maximum_functions,
      },
      allow_memory_grow = false,
      allow_start = false,
      duplicate_custom_sections = "reject",
      producer_metadata = "recorded-only",
      severity = {
        malformed = "error",
        disallowed_kind = "error",
        disallowed_import = "error",
        disallowed_world = "error",
        resource_limit = "error",
        growth = "error",
        start = "error",
        duplicate_metadata = "error",
      },
      verification_role = "recorded_only",
      non_claims = [
        "Wasm artifact evidence records bounded static facts and does not prove execution correctness",
        "Matching normalized interfaces do not prove semantic equivalence or identify one implementation",
        "Static validation does not prove sandbox effectiveness, capability authority, provenance truth, release eligibility, or whole-system safety",
        "Producer metadata is recorded input and is not provenance truth",
      ],
    },
  ],
} | contracts.Registry
