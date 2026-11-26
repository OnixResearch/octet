# Negative fixture contract proving a required formal_proof clause stays unmet when only structural linkage is supplied.
{
  schema_version = "octet-requirement-contract/v1",
  contract_id = "octet.structural_linkage.pilot.negative_formal",
  reviewed_specification = {
    path = "standards/structural-linkage/spec.md",
    blake3 = "09b6db8b5874245230f5335fbef7a7bad45cc358a57d7f9df4d9f49b58c1d9bf",
  },
  clauses = [
    {
      clause_id = "octet.structural_linkage.pilot.checked_add",
      statement = "Requires formal proof for checked_add; structural linkage alone must leave this clause unmet.",
      required = true,
      disposition = "formal_proof",
      checked_scope = ["pilot/fn:checked_add#0"],
      expected_evidence_tier = "formal_proof",
      allowed_evidence_ids = ["cairn.pilot.checked_add"],
      allowed_source_requirement_ids = ["octet.structural_linkage.pilot.checked_add"],
      required_evidence_classes = ["proof_card"],
      required_negative_controls = [],
      required_bindings = {
        source_ids = ["standards/structural-linkage/fixtures/target.rs"],
        policy_ids = ["cairn-structural-trace-v2"],
        tool_identity_hashes = [],
        artifact_hashes = [],
        checked_scope = ["pilot/fn:checked_add#0"],
        evidence_tier = "formal_proof",
      },
      source_bindings = [
        {
          locator = "standards/structural-linkage/fixtures/target.rs",
          blake3 = "bb1f75d6c43eefe8bd026e244f6d14db95e4f17d60bbd439c90ce21a312ab3c8",
        },
      ],
    },
  ],
  assumptions = [],
  exclusions = [],
  non_claims = [
    "negative control proving structural linkage cannot satisfy formal_proof obligations",
  ],
}
