# Effect-extraction pilot requirement contract (#18).
#
# Export deterministic JSON with:
#   nickel export --format json standards/requirements/effect-extraction/contract.ncl \
#     | jq -S . > standards/requirements/effect-extraction/contract.json
let contracts = import "./contracts.ncl" in
let spec_h = "b524ad5fc87548b965997b67c2e7e2dce833c0e00dfc6eff85756680e83502a7" in
let orig_h = "d285236ecacc09bf14375ad5a000bb7b752da2ef83e64a632df70459ed34a2ff" in
let core_h = "3e8ada75d8619b3b7b302785f133dc86511fda8e6cf7103a65bb11d79e024275" in
let drv_h = "3c0b52b71e6e97a4b0018e9c145b6cc06706756adb6ebee43050cd9438569e0d" in
let lock_h = "5274c486e8fc3004e94cac956e1533c0e558eb17eed5605f66a46290c740c8cf" in
let tool_h = "50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8" in
{
  schema_version = "octet-requirement-contract/v1",
  contract_id = "octet.effect_extraction.pilot",
  reviewed_specification = {
    path = "standards/requirements/effect-extraction/spec.md",
    blake3 = spec_h,
  },
  clauses = [
    {
      clause_id = "extraction.subjects",
      statement = "Original implementation, extracted core, effect driver, and specification bytes are frozen.",
      required = true,
      disposition = "empirical_check",
      checked_scope = ["pilot:effect-extraction:subjects"],
      expected_evidence_tier = "empirical_check",
      allowed_evidence_ids = ["check.subjects"],
      source_bindings = [
        {
          locator = "standards/requirements/fixtures/effect-extraction/original.rs",
          blake3 = orig_h,
        },
        {
          locator = "standards/requirements/fixtures/effect-extraction/extracted_core.rs",
          blake3 = core_h,
        },
        {
          locator = "standards/requirements/fixtures/effect-extraction/driver.rs",
          blake3 = drv_h,
        },
      ],
    },
    {
      clause_id = "extraction.core_effects",
      statement = "Extracted core satisfies the semantic MIR effect-closure profile with zero unknown effects.",
      required = true,
      disposition = "empirical_check",
      checked_scope = ["pilot:effect-extraction:core"],
      expected_evidence_tier = "empirical_check",
      allowed_evidence_ids = ["check.core_effects"],
      source_bindings = [
        {
          locator = "standards/requirements/fixtures/effect-extraction/extracted_core.rs",
          blake3 = core_h,
        },
      ],
    },
    {
      clause_id = "extraction.behavior_preservation",
      statement = "Original implementation and driver-composed extracted core satisfy the reviewed observation contract.",
      required = true,
      disposition = "formal_proof",
      checked_scope = ["pilot:effect-extraction:core"],
      expected_evidence_tier = "formal_proof",
      allowed_evidence_ids = ["proof.preservation"],
      allowed_source_requirement_ids = ["extraction.behavior_preservation"],
      source_bindings = [
        {
          locator = "standards/requirements/fixtures/effect-extraction/driver.rs",
          blake3 = drv_h,
        },
      ],
    },
    {
      clause_id = "extraction.correspondence",
      statement = "Source-to-backend correspondence connects the exact checked Rust core and proof model.",
      required = true,
      disposition = "empirical_check",
      checked_scope = ["pilot:effect-extraction:core"],
      expected_evidence_tier = "empirical_check",
      allowed_evidence_ids = ["check.correspondence"],
      source_bindings = [
        {
          locator = "standards/requirements/fixtures/effect-extraction/extracted_core.rs",
          blake3 = core_h,
        },
      ],
    },
    {
      clause_id = "extraction.freshness",
      statement = "All source snapshots, semantic summaries, and proof receipts remain current.",
      required = true,
      disposition = "empirical_check",
      checked_scope = ["pilot:effect-extraction:subjects"],
      expected_evidence_tier = "empirical_check",
      allowed_evidence_ids = ["check.subjects"],
      source_bindings = [
        {
          locator = "standards/requirements/fixtures/effect-extraction/original.rs",
          blake3 = orig_h,
        },
      ],
    },
  ],
  non_claims = [
    "bounded sequential extraction pilot only; does not prove arbitrary Rust, concurrency, or whole-program correctness",
    "evaluated against retained fixture artifacts; does not prove verifier or rustc soundness",
  ],
  effect_extraction = ({
    schema_version = "octet-effect-extraction-profile/v1",
    profile_id = "effect-extraction-sequential-v1",
    pilot_id = "compiler-lab-194",
    subjects = {
      original_implementation = {
        locator = "standards/requirements/fixtures/effect-extraction/original.rs",
        blake3 = orig_h,
        role = "original",
      },
      extracted_core = {
        locator = "standards/requirements/fixtures/effect-extraction/extracted_core.rs",
        blake3 = core_h,
        role = "core",
      },
      effect_driver = {
        locator = "standards/requirements/fixtures/effect-extraction/driver.rs",
        blake3 = drv_h,
        role = "driver",
      },
      specification = {
        locator = "standards/requirements/effect-extraction/spec.md",
        blake3 = spec_h,
        role = "spec",
      },
      dependency_closure_blake3 = lock_h,
      semantic_profile_id = "semantic-mir-core",
      target_triple = "x86_64-unknown-linux-gnu",
      toolchain_blake3 = tool_h,
    },
    observation_contract = {
      relation = "observational-equivalence",
      return_values = "exact-value-equality",
      explicit_errors = "preserved-error-variant",
      operation_order_multiplicity = "exact-ordered-trace",
      externally_visible_state = "modeled-state-equality",
      destruction_treatment = "modeled-normal-and-unwind-drop",
      panic_unwind_treatment = "preserved-or-excluded",
      allocation_treatment = "profile-local",
      termination_treatment = "total-bounded-sequential",
      excluded_concurrency = "no-threads-atomics-async",
      input_domain = "u32",
      preconditions = "all-u32-inputs",
      causal_environment = "deterministic-sequential-driver",
    },
    clauses = {
      subjects_clause_id = "extraction.subjects",
      core_effects_clause_id = "extraction.core_effects",
      behavior_preservation_clause_id = "extraction.behavior_preservation",
      correspondence_clause_id = "extraction.correspondence",
      freshness_clause_id = "extraction.freshness",
      optional_normal_form_clause = null,
      effect_independence_clause_id = null,
    },
    approved_assumptions = [],
  } | contracts.EffectExtractionProfile),
}
