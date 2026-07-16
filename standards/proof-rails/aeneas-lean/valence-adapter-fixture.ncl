let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let TrueOnly = std.contract.from_predicate (fun value => value == true) in
let RequiredStageKinds = [
  "rust_source",
  "charon_extraction",
  "aeneas_translation",
  "generated_lean",
  "theorem",
  "lean_check",
] in
let CompleteStageInventory = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value == std.array.length RequiredStageKinds
  && std.array.all (fun kind =>
    std.array.any (fun stage => stage.stage_kind == kind) value
  ) RequiredStageKinds
) in
let Stage = {
  stage_kind | NonEmptyString,
  artifact | NonEmptyString,
  blake3 | Digest,
  evidence_role | OneOf ["recorded_only", "boundary", "property_candidate"],
} in
let Fixture = {
  schema_version | OneOf ["octet-aeneas-lean-valence-adapter/v1"],
  adapter_target | OneOf ["valence.stack-provenance.adapter-input"],
  provider_id | OneOf ["aeneas-lean"],
  octet_export | {
    artifact_directory | NonEmptyString,
    artifact_verification_status | OneOf ["valid"],
    artifact_verification_receipt | NonEmptyString,
    proof_receipt_id | NonEmptyString,
    proof_receipt_status | OneOf ["passed"],
    exported_after_verification | TrueOnly,
  },
  provider_profile | NonEmptyString,
  package_lock | NonEmptyString,
  stages | CompleteStageInventory | Array Stage,
  property_candidate | {
    theorem_id | NonEmptyString,
    requirement_id | NonEmptyString,
    receipt_artifact | NonEmptyString,
    evidence_tier | OneOf ["formal_proof"],
    owner | OneOf ["octet"],
  },
  non_claims | Array NonEmptyString,
} in
{
  schema_version = "octet-aeneas-lean-valence-adapter/v1",
  adapter_target = "valence.stack-provenance.adapter-input",
  provider_id = "aeneas-lean",
  octet_export = {
    artifact_directory = "artifacts/aeneas-lean-fixture",
    artifact_verification_status = "valid",
    artifact_verification_receipt = "artifacts/aeneas-lean-fixture/verification.json",
    proof_receipt_id = "octet.aeneas_lean_proof_rail.fixture#receipt-0",
    proof_receipt_status = "passed",
    exported_after_verification = true,
  },
  provider_profile = "standards/proof-rails/aeneas-lean/generated/profile.json",
  package_lock = "package-lock.json",
  stages = [
    {
      stage_kind = "rust_source",
      artifact = "src/verified/addition.rs",
      blake3 = "b3:0000000000000000000000000000000000000000000000000000000000000000",
      evidence_role = "recorded_only",
    },
    {
      stage_kind = "charon_extraction",
      artifact = "stages/charon-options.json",
      blake3 = "b3:1111111111111111111111111111111111111111111111111111111111111111",
      evidence_role = "boundary",
    },
    {
      stage_kind = "aeneas_translation",
      artifact = "stages/translation.json",
      blake3 = "b3:2222222222222222222222222222222222222222222222222222222222222222",
      evidence_role = "recorded_only",
    },
    {
      stage_kind = "generated_lean",
      artifact = "generated/Add.lean",
      blake3 = "b3:3333333333333333333333333333333333333333333333333333333333333333",
      evidence_role = "recorded_only",
    },
    {
      stage_kind = "theorem",
      artifact = "theorems/addition.lean",
      blake3 = "b3:4444444444444444444444444444444444444444444444444444444444444444",
      evidence_role = "boundary",
    },
    {
      stage_kind = "lean_check",
      artifact = "proofs/aeneas.receipt",
      blake3 = "b3:5555555555555555555555555555555555555555555555555555555555555555",
      evidence_role = "property_candidate",
    },
  ],
  property_candidate = {
    theorem_id = "aeneas.fixture.safe_add",
    requirement_id = "octet.aeneas_lean_proof_rail.theorem_adequacy",
    receipt_artifact = "proofs/aeneas.receipt",
    evidence_tier = "formal_proof",
    owner = "octet",
  },
  non_claims = [
    "Valence consumes this adapter input for identity and linkage only",
    "does not prove Aeneas or Charon translation soundness",
    "does not prove Lean or verifier soundness",
    "does not prove whole-program correctness",
    "does not establish release eligibility",
  ],
} | Fixture
