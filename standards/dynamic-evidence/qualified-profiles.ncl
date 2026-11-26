let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let PositiveInt = std.contract.from_predicate (fun value => std.is_number value && value >= 1 && value % 1 == 0) in
let NonEmptyStrings = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.all (fun item => std.is_string item && std.string.length item > 0) value
) in
let MatrixCellSpec = {
  cell_id | NonEmptyString,
  tool | NonEmptyString,
  target_triple | NonEmptyString,
  borrow_model | String | optional,
  seed | Number | optional,
  preemption_bound | Number | optional,
  required | Bool,
} in
let MiriSpec = {
  toolchain_pin | NonEmptyString,
  target_triple | NonEmptyString,
  borrow_model | NonEmptyString,
  check_provenance | Bool,
  isolation | NonEmptyString,
  seeds | Array PositiveInt,
  miri_flags | NonEmptyStrings,
} in
let LoomSpec = {
  model_identity | NonEmptyString,
  max_threads | PositiveInt,
  max_branches | PositiveInt,
  max_permutations | PositiveInt,
  preemption_bound | PositiveInt,
  stopping_rules | NonEmptyString,
  expected_model_entry_marker | NonEmptyString,
  expected_model_completion_marker | NonEmptyString,
  rustflags | NonEmptyString,
  encoded_rustflags | NonEmptyString,
} in
let Limits = {
  timeout_seconds | PositiveInt,
  max_output_bytes | PositiveInt,
  max_source_files | PositiveInt,
  max_input_bytes | PositiveInt,
  poll_interval_ms | PositiveInt,
  teardown_timeout_ms | PositiveInt,
} in
let Profile = {
  profile_id | NonEmptyString,
  tool | std.contract.from_predicate (fun value => value == "miri" || value == "loom"),
  evidence_tier | std.contract.from_predicate (fun value => value == "transcript_assertion"),
  package | NonEmptyString,
  target | NonEmptyString,
  cargo_features | Array String,
  cargo_cfg | Array String,
  command_argv | NonEmptyStrings,
  test_inventory | NonEmptyStrings,
  expected_workload | NonEmptyString,
  limits | Limits,
  matrix_cells | std.contract.from_predicate (fun value => std.is_array value && std.array.length value > 0),
  miri | MiriSpec | optional,
  loom | LoomSpec | optional,
  non_claims | NonEmptyStrings,
} in
let Registry = {
  schema_version | std.contract.from_predicate (fun value => value == "octet-dynamic-qualified-profiles/v1"),
  profiles | Array Profile,
} in
{
  schema_version = "octet-dynamic-qualified-profiles/v1",
  profiles = [
    {
      profile_id = "miri-unsafe-boundary-qualified",
      tool = "miri",
      evidence_tier = "transcript_assertion",
      package = "octet-dynamic-pilot",
      target = "lib",
      cargo_features = [],
      cargo_cfg = ["miri"],
      command_argv = [
        "cargo",
        "miri",
        "test",
        "--locked",
        "--offline",
        "-p",
        "octet-dynamic-pilot",
        "--lib",
        "--",
        "--test-threads=1",
      ],
      test_inventory = [
        "tests::checked_slice_window_sound",
        "tests::raw_pointer_read_aligned",
      ],
      expected_workload = "Execute deterministic unsafe pointer/slice boundary tests under Miri with strict provenance and Tree Borrows enabled",
      limits = {
        timeout_seconds = 120,
        max_output_bytes = 262144,
        max_source_files = 64,
        max_input_bytes = 524288,
        poll_interval_ms = 25,
        teardown_timeout_ms = 5000,
      },
      matrix_cells = [
        {
          cell_id = "miri-x86_64-tree-borrows-seed1",
          tool = "miri",
          target_triple = "x86_64-unknown-linux-gnu",
          borrow_model = "tree-borrows",
          seed = 1,
          required = true,
        },
      ],
      miri = {
        toolchain_pin = "1.98.0-nightly",
        target_triple = "x86_64-unknown-linux-gnu",
        borrow_model = "tree-borrows",
        check_provenance = true,
        isolation = "strict",
        seeds = [1],
        miri_flags = [
          "-Zmiri-strict-provenance",
          "-Zmiri-tree-borrows",
          "-Zmiri-seed=1",
        ],
      },
      non_claims = [
        "Miri checks executed Rust abstract-machine paths under declared seeds, borrow rules, and targets only; it does not prove whole-program soundness or unexecuted paths",
        "Finite seed and test-inventory dynamic evidence does not replace formal proof or release eligibility",
      ],
    },
    {
      profile_id = "loom-concurrency-qualified",
      tool = "loom",
      evidence_tier = "transcript_assertion",
      package = "octet-dynamic-pilot",
      target = "lib",
      cargo_features = [],
      cargo_cfg = ["loom"],
      command_argv = [
        "cargo",
        "test",
        "--locked",
        "--offline",
        "-p",
        "octet-dynamic-pilot",
        "--lib",
        "loom_models",
        "--",
        "--test-threads=1",
      ],
      test_inventory = [
        "loom_models::bounded_handoff_preserves_order",
      ],
      expected_workload = "Explore bounded thread interleavings for the two-thread handoff model with explicit preemption and branch caps",
      limits = {
        timeout_seconds = 120,
        max_output_bytes = 262144,
        max_source_files = 64,
        max_input_bytes = 524288,
        poll_interval_ms = 25,
        teardown_timeout_ms = 5000,
      },
      matrix_cells = [
        {
          cell_id = "loom-x86_64-preemption2-t3",
          tool = "loom",
          target_triple = "x86_64-unknown-linux-gnu",
          preemption_bound = 2,
          required = true,
        },
      ],
      loom = {
        model_identity = "loom_models::bounded_handoff_preserves_order",
        max_threads = 3,
        max_branches = 1000,
        max_permutations = 256,
        preemption_bound = 2,
        stopping_rules = "fail-on-first-counterexample-or-exceeded-preemption-bound",
        expected_model_entry_marker = "loom::model",
        expected_model_completion_marker = "loom_model_completed",
        rustflags = "--cfg loom",
        encoded_rustflags = "--cfg\x1floom",
      },
      non_claims = [
        "Loom explores bounded schedules under configured preemption, branch, and thread caps only; it does not prove correctness under arbitrary OS scheduling or unbounded concurrency",
        "Finite schedule dynamic evidence does not prove whole-program soundness or release eligibility",
      ],
    },
  ],
} | Registry
