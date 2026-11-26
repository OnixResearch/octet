let Contract = import "consumer-contract.ncl" in
let target_pointer_width_bytes = 8 in
let provider_timeout_ms = 30000 in
let bounded_log_bytes = 4096 in
let snapshot_mebibytes = 8 in
let bytes_per_kibibyte = 1024 in
let kibibytes_per_mebibyte = 1024 in
let configured_snapshot_max_bytes = snapshot_mebibytes * bytes_per_kibibyte * kibibytes_per_mebibyte in
let configured_item_max_count = 32 in
let configured_edge_max_count = 128 in
{
  schema_version = "octet-charon-semantic-consumer-profile/v1",
  profile_id = "octet-charon-semantic-consumer-trait-helper-v1",
  provider = {
    package = "octet-aeneas-lean-provider-closure",
    profile_id = "octet-aeneas-lean-0.1.0-bc2eb6f",
    profile_blake3 = "cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625",
    compatibility_matrix_id = "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16",
    compatibility_matrix_blake3 = "403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5",
    charon_revision = "cb50ff16b9f1066b8a97dc06da704de2da2fa41c",
    executable_env = "OCTET_AENEAS_CHARON",
    network_sandbox_env = "OCTET_AENEAS_NETWORK_SANDBOX",
  },
  source = {
    root = "standards/charon-semantic/fixtures/consumer/trait-helper",
    manifest = {
      path = "Cargo.toml",
      identity = "b3:e546dd480068168ebafd2b96f4df469499694f8da0850361f5f8ab9f6d2347f2",
    },
    lockfile = {
      path = "Cargo.lock",
      identity = "b3:2219a2a265930b1b768b4b8406426c70d0f1db376b4e29fc0c56aaa97afce237",
    },
    crate_name = "trait_helper_consumer",
    source_files = [
      {
        path = "src/helper.rs",
        identity = "b3:aafee6055e3ca6db6a17a608f80ab969372c3a9fd4d8f349a54c4a5b8c06f6f9",
      },
      {
        path = "src/lib.rs",
        identity = "b3:64b1ea7e8b1e0f902745c18a1cbba86abb4914441b1a489088205f25b9afb4af",
      },
    ],
    selected_roots = [
      {
        qualified_name = "trait_helper_consumer::scale_twice",
        kind = "function",
        source_file = "src/lib.rs",
        source_identity = "b3:64b1ea7e8b1e0f902745c18a1cbba86abb4914441b1a489088205f25b9afb4af",
        begin_line = 3,
        end_line = 6,
      },
    ],
    broad_selection_allowed = false,
  },
  extraction = {
    preset = "Aeneas",
    has_errors = false,
    skip_borrowck = false,
    borrow_check_preserved = true,
    target_triple = "x86_64-unknown-linux-gnu",
    pointer_width_bytes = target_pointer_width_bytes,
    endianness = "little",
    serialization_format = "llbc-json-v1",
    no_default_features = false,
    features = [],
    cfg = [],
    environment_allowlist = ["PATH", "CHARON_TOOLCHAIN_IS_IN_PATH", "LD_LIBRARY_PATH", "HOME"],
  },
  fact_vocabulary = [
    "selected-item",
    "source-span",
    "local-call",
    "item-dependency",
    "trait-resolution",
    "control-flow-shape",
    "call-cycle",
    "opacity",
    "unsupported-marker",
    "target-metadata",
  ],
  effect_candidate_vocabulary = ["panic", "drop", "unsafe-boundary", "unknown-call"],
  limits = {
    timeout_ms = provider_timeout_ms,
    stdout_max_bytes = bounded_log_bytes,
    stderr_max_bytes = bounded_log_bytes,
    snapshot_max_bytes = configured_snapshot_max_bytes,
    item_max_count = configured_item_max_count,
    edge_max_count = configured_edge_max_count,
  },
  artifact_roles = [
    "raw-charon-snapshot",
    "normalized-semantic-projection",
    "bounded-semantic-analysis",
    "source-token-comparison",
    "recorded-only-receipt",
  ],
  references = [
    {
      repository = "https://github.com/AeneasVerif/mechanized-llbc",
      revision = "8a9b8bb3823ceceb47026837d23baea0934dfb27",
      license = "Apache-2.0",
      source_sentinel = "README.md:Mechanized LLBC",
      use = "semantic-foundation review corpus only",
      recorded_only = true,
    },
    {
      repository = "https://github.com/AeneasVerif/charon-rudra",
      revision = "0923bf7f21d4eb234d14bce86afc9690d0a4d00e",
      license = "Apache-2.0",
      source_sentinel = "src/main.rs:crate::rudra::lib::analyze",
      use = "historical analysis-over-Charon fixture design only",
      recorded_only = true,
    },
  ],
  verification_role = "recorded_only",
  non_claims = [
    "does not prove Charon or rustc correctness",
    "does not prove complete Rust support or complete runtime reachability",
    "does not prove effect absence, semantic equivalence, or whole-program correctness",
    "does not produce formal-proof, property, or release-ready evidence",
    "does not transfer mechanized-llbc or charon-rudra proof, implementation, or analysis authority",
    "does not make Valence or Kamacite a compiler IR or semantic-analysis authority",
  ],
} | Contract.ConsumerProfile
