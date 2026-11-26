let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let PositiveInt = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0 && std.number.is_integer value
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let ConfigModeContract = OneOf [
  "plain-compiler",
  "lint-collection",
  "architecture-collection",
  "selected-value-flow",
] in
{
  SelfProfileConfig = {
    schema_version | OneOf ["octet-compiler-self-profile/v1"],
    profile_id | NonEmptyString,
    rustc_format_version | PositiveInt,
    self_profile_events | NonEmptyString,
    configurations | NonEmptyArray | Array {
      config_id | ConfigModeContract,
      description | NonEmptyString,
      claim_coverage_note | NonEmptyString,
      uses_octet_driver | Bool,
      enables_architecture_collection | Bool,
      enables_value_flow | Bool,
    },
    budgets | {
      max_mm_profdata_bytes | PositiveInt,
      max_events_per_unit | PositiveInt,
      max_units | PositiveInt,
      phase_record_budget | PositiveInt,
    },
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
}
