let NonEmptyString = std.contract.from_predicate (fun value => std.string.length value > 0) in
let Status = std.contract.from_predicate (fun value => value == "passed" || value == "failed" || value == "blocked") in
let Evidence = {
  path | NonEmptyString,
  blake3 | NonEmptyString,
} in
let Stage = {
  name | NonEmptyString,
  status | Status,
} in
{
  AcceptanceReceipt = {
    schema_version | std.contract.from_predicate (fun value => value == "tigerstyle-acceptance-receipt/v1"),
    run_id | NonEmptyString,
    started_at | NonEmptyString,
    finished_at | NonEmptyString,
    tool | NonEmptyString,
    tool_version | NonEmptyString,
    git_revision | NonEmptyString,
    config_digest | NonEmptyString,
    inputs | Array NonEmptyString,
    stages | Array Stage,
    artifacts | Array NonEmptyString,
    status | Status,
    warnings | Array String | default = [],
    errors | Array String | default = [],
    verification_commands | Array NonEmptyString,
    verification_evidence | Array Evidence,
    raw_log_path | String | default = "",
  },
}
