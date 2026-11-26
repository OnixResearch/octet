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
let SemanticEffectContract = OneOf [
  "alloc",
  "panic",
  "unsafe",
  "clock",
  "random",
  "fs",
  "net",
  "spawn",
  "lock",
  "ffi",
  "external-mutation",
  "interior-mutation",
  "unwind-drop",
  "unknown",
] in
let UnwindModeContract = OneOf ["modeled-cleanup", "reject-unwind"] in
let ClaimTierContract = OneOf ["semantic-mir-effect-closure"] in
let CompilerStageContract = OneOf ["rustc-mir-optimized"] in
let BudgetsContract = {
  max_bodies | PositiveInt,
  max_blocks_per_body | PositiveInt,
  max_steps | PositiveInt,
  max_rounds | PositiveInt,
  call_edge_limit | PositiveInt,
} in
let SubsetContract = {
  subset_id | NonEmptyString,
  included_constructs | NonEmptyArray | Array NonEmptyString,
  excluded_constructs | NonEmptyArray | Array NonEmptyString,
} in
{
  SemanticEffect = SemanticEffectContract,
  UnwindMode = UnwindModeContract,
  SemanticEffectProfile = {
    schema_version | OneOf ["octet-semantic-effect-profile/v1"],
    profile_id | NonEmptyString,
    claim_tier | ClaimTierContract,
    compiler_stage | CompilerStageContract,
    unwind_mode | UnwindModeContract,
    supported_subset | SubsetContract,
    effect_vocabulary | NonEmptyArray | Array SemanticEffectContract,
    allowed_effects | Array SemanticEffectContract,
    denied_effects | NonEmptyArray | Array SemanticEffectContract,
    budgets | BudgetsContract,
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
}
