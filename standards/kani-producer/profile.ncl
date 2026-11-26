let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let PositiveInt = std.contract.from_predicate (fun value => std.is_number value && value >= 1 && value % 1 == 0) in
let NonEmptyStrings = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.all (fun item => std.is_string item && std.string.length item > 0) value
) in
let HarnessSpec = {
  name | NonEmptyString,
  kind | std.contract.from_predicate (fun value => value == "panic_freedom" || value == "cover_witness"),
  unwind | PositiveInt,
  cover_properties | Array String,
} in
let KaniPin = {
  kani_version | NonEmptyString,
  provider_revision | NonEmptyString,
  closure_identity | NonEmptyString,
  rustc_release | NonEmptyString,
  cbmc_version | NonEmptyString,
  solver_mode | std.contract.from_predicate (fun value => value == "cbmc_builtin" || value == "external_solver"),
  solver_identity | NonEmptyString,
  export_flags | NonEmptyStrings,
} in
let Profile = {
  schema_version | std.contract.from_predicate (fun value => value == "octet-kani-producer-profile/v1"),
  profile_id | NonEmptyString,
  evidence_tier | std.contract.from_predicate (fun value => value == "transcript_assertion"),
  package | NonEmptyString,
  target | NonEmptyString,
  cargo_features | Array String,
  cargo_cfg | Array String,
  kani_pin | KaniPin,
  command_prefix | NonEmptyStrings,
  harnesses | std.contract.from_predicate (fun value => std.is_array value && std.array.length value > 0) | Array HarnessSpec,
  default_unwind | PositiveInt,
  require_unwind_assertions | Bool,
  require_reachable_cover | Bool,
  require_assumption_review | Bool,
  limits | {
    timeout_seconds | PositiveInt,
    max_output_bytes | PositiveInt,
    max_export_bytes | PositiveInt,
    max_source_files | PositiveInt,
    poll_interval_ms | PositiveInt,
    teardown_timeout_ms | PositiveInt,
  },
  non_claims | NonEmptyStrings,
} in
{
  schema_version = "octet-kani-producer-profile/v1",
  profile_id = "kani-0.68.0-panic-freedom",
  evidence_tier = "transcript_assertion",
  package = "octet-kani-boundary-pilot",
  target = "lib",
  cargo_features = [],
  cargo_cfg = ["kani"],
  kani_pin = {
    kani_version = "0.68.0",
    provider_revision = "kani-0.68.0-release",
    closure_identity = "nix:kani-0.68.0-x86_64-unknown-linux-gnu",
    rustc_release = "1.87.0-nightly",
    cbmc_version = "6.5.0",
    solver_mode = "cbmc_builtin",
    solver_identity = "cbmc-builtin-cadical",
    export_flags = ["-Z", "unstable-options", "--export-json"],
  },
  command_prefix = [
    "cargo",
    "kani",
    "--locked",
    "--offline",
    "-p",
    "octet-kani-boundary-pilot",
    "--lib",
  ],
  harnesses = [
    {
      name = "check_checked_window_bounds",
      kind = "panic_freedom",
      unwind = 8,
      cover_properties = ["cover_valid_window_reachable"],
    },
  ],
  default_unwind = 8,
  require_unwind_assertions = true,
  require_reachable_cover = true,
  require_assumption_review = true,
  limits = {
    timeout_seconds = 180,
    max_output_bytes = 524288,
    max_export_bytes = 2097152,
    max_source_files = 64,
    poll_interval_ms = 25,
    teardown_timeout_ms = 5000,
  },
  non_claims = [
    "Kani checks bounded symbolic execution over declared harnesses and unwind bounds only; it does not prove whole-program correctness, verifier soundness, or release eligibility",
    "Native Kani producer execution records bounded transcript_assertion evidence gated by separate human assumption review and reachable cover witnesses, never formal_proof",
  ],
} | Profile
