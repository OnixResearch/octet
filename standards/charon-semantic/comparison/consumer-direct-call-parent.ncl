let NonEmptyString = std.contract.from_predicate (fun value => std.string.is_match "^.+$" value) in
let B3Identity = std.contract.from_predicate (fun value => std.string.is_match "^b3:[0-9a-f]{64}$" value) in
let Fact = {
  item | NonEmptyString,
  fact | NonEmptyString,
  value | NonEmptyString,
} in
let ComparisonParent = {
  schema_version | std.contract.Equal "octet-charon-semantic-comparison-parent/v1",
  authority | std.contract.Equal "source-token",
  identity | B3Identity,
  source_identity | B3Identity,
  facts | Array Fact,
} in
{
  schema_version = "octet-charon-semantic-comparison-parent/v1",
  authority = "source-token",
  identity = "b3:23e41fc9fa5d85379bac551522134416b0478cbaf19d67d35e69a6f0b53b906e",
  source_identity = "b3:fd772d58dba97f78ae03dda54acd2a62ceaa191711233f20a0ccf1f9bdb08cd1",
  facts = [
    { item = "direct_call_consumer::combine_steps", fact = "item-presence", value = "present" },
    { item = "direct_call_consumer::combine_steps", fact = "control-flow", value = "structured" },
    { item = "direct_call_consumer::combine_steps", fact = "local-call", value = "direct_call_consumer::helper::bounded_step" },
    { item = "direct_call_consumer::helper::bounded_step", fact = "item-presence", value = "present" },
    { item = "direct_call_consumer::helper::bounded_step", fact = "effect-candidate", value = "panic" },
  ],
} | ComparisonParent
