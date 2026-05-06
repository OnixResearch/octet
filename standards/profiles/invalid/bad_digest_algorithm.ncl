let contracts = import "../contracts.ncl" in
{
  schema_version = "tigerstyle-run-profile/v1",
  profile_id = "bad-profile",
  source_paths = ["docs/standards/source"],
  artifact_out_dir = "build/tigerstyle-artifacts",
  receipt_out_dir = "build/tigerstyle-receipts",
  config_digest_algorithm = "sha256",
  enabled_policy_families = ["architecture"],
} | contracts.RunProfile
