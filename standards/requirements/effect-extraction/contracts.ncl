let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 64
) in
let SubjectRef = {
  locator | NonEmptyString,
  blake3 | Blake3Hex,
  role | NonEmptyString,
} in
{
  EffectExtractionProfile = {
    schema_version | std.contract.from_predicate (fun v => v == "octet-effect-extraction-profile/v1"),
    profile_id | NonEmptyString,
    pilot_id | NonEmptyString,
    subjects | {
      original_implementation | SubjectRef,
      extracted_core | SubjectRef,
      effect_driver | SubjectRef,
      specification | SubjectRef,
      dependency_closure_blake3 | NonEmptyString,
      semantic_profile_id | NonEmptyString,
      target_triple | NonEmptyString,
      toolchain_blake3 | NonEmptyString,
    },
    observation_contract | {
      relation | NonEmptyString,
      return_values | NonEmptyString,
      explicit_errors | NonEmptyString,
      operation_order_multiplicity | NonEmptyString,
      externally_visible_state | NonEmptyString,
      destruction_treatment | NonEmptyString,
      panic_unwind_treatment | NonEmptyString,
      allocation_treatment | NonEmptyString,
      termination_treatment | NonEmptyString,
      excluded_concurrency | NonEmptyString,
      input_domain | NonEmptyString,
      preconditions | NonEmptyString,
      causal_environment | NonEmptyString,
    },
    clauses | {
      subjects_clause_id | NonEmptyString,
      core_effects_clause_id | NonEmptyString,
      behavior_preservation_clause_id | NonEmptyString,
      correspondence_clause_id | NonEmptyString,
      freshness_clause_id | NonEmptyString,
      optional_normal_form_clause | Dyn,
      effect_independence_clause_id | Dyn,
    },
    approved_assumptions | Array NonEmptyString,
    optional_normal_form | default = null,
    optional_effect_independence | default = null,
  },
}
