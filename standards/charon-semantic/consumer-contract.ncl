let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let HexDigest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let B3Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let Revision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let FileDigest = {
  path | NonEmptyString,
  identity | B3Identity,
} in
let SelectedRoot = {
  qualified_name | NonEmptyString,
  kind | std.contract.Equal "function",
  source_file | NonEmptyString,
  source_identity | B3Identity,
  begin_line | PositiveNumber,
  end_line | PositiveNumber,
} in
let Reference = {
  repository | NonEmptyString,
  revision | Revision,
  license | std.contract.Equal "Apache-2.0",
  source_sentinel | NonEmptyString,
  use | NonEmptyString,
  recorded_only | std.contract.Equal true,
} in
{
  ConsumerProfile = {
    schema_version | std.contract.Equal "octet-charon-semantic-consumer-profile/v1",
    profile_id | NonEmptyString,
    provider | {
      package | std.contract.Equal "octet-aeneas-lean-provider-closure",
      profile_id | std.contract.Equal "octet-aeneas-lean-0.1.0-bc2eb6f",
      profile_blake3 | HexDigest,
      compatibility_matrix_id | std.contract.Equal "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16",
      compatibility_matrix_blake3 | HexDigest,
      charon_revision | std.contract.Equal "cb50ff16b9f1066b8a97dc06da704de2da2fa41c",
      executable_env | std.contract.Equal "OCTET_AENEAS_CHARON",
      network_sandbox_env | std.contract.Equal "OCTET_AENEAS_NETWORK_SANDBOX",
    },
    source | {
      root | NonEmptyString,
      manifest | FileDigest,
      lockfile | FileDigest,
      crate_name | NonEmptyString,
      source_files | Array FileDigest,
      selected_roots | Array SelectedRoot,
      broad_selection_allowed | std.contract.Equal false,
    },
    extraction | {
      preset | std.contract.Equal "Aeneas",
      has_errors | std.contract.Equal false,
      skip_borrowck | std.contract.Equal false,
      borrow_check_preserved | std.contract.Equal true,
      target_triple | std.contract.Equal "x86_64-unknown-linux-gnu",
      pointer_width_bytes | std.contract.Equal 8,
      endianness | std.contract.Equal "little",
      serialization_format | std.contract.Equal "llbc-json-v1",
      no_default_features | Bool,
      features | Array NonEmptyString,
      cfg | Array NonEmptyString,
      environment_allowlist | Array NonEmptyString,
    },
    fact_vocabulary | Array NonEmptyString,
    effect_candidate_vocabulary | Array NonEmptyString,
    limits | {
      timeout_ms | PositiveNumber,
      stdout_max_bytes | PositiveNumber,
      stderr_max_bytes | PositiveNumber,
      snapshot_max_bytes | PositiveNumber,
      item_max_count | PositiveNumber,
      edge_max_count | PositiveNumber,
    },
    artifact_roles | Array NonEmptyString,
    references | Array Reference,
    verification_role | std.contract.Equal "recorded_only",
    non_claims | Array NonEmptyString,
  },
}
