# Reviewed requirement-to-evidence contract for Cairn structural linkage pilot.
{
  schema_version = "octet-requirement-contract/v1",
  contract_id = "octet.structural_linkage.pilot",
  reviewed_specification = {
    path = "standards/structural-linkage/spec.md",
    blake3 = "09b6db8b5874245230f5335fbef7a7bad45cc358a57d7f9df4d9f49b58c1d9bf",
  },
  clauses = [
    {
      clause_id = "octet.structural_linkage.pilot.checked_add",
      statement = "Parser-resolved Rust item anchor binds checked_add to its reviewed requirement without promoting proof status.",
      required = true,
      disposition = "structural_linkage",
      checked_scope = ["pilot/fn:checked_add#0"],
      expected_evidence_tier = "structural_linkage",
      allowed_evidence_ids = ["cairn.pilot.checked_add"],
      allowed_source_requirement_ids = ["octet.structural_linkage.pilot.checked_add"],
      required_evidence_classes = ["structural_linkage"],
      required_negative_controls = [],
      required_bindings = {
        source_ids = ["standards/structural-linkage/fixtures/target.rs"],
        policy_ids = ["cairn-structural-trace-v2"],
        tool_identity_hashes = [
          "759c26163339c2a9813cd969f8b11004bc44f97d122029490b0dc4fc3a448c69",
        ],
        artifact_hashes = [],
        checked_scope = ["pilot/fn:checked_add#0"],
        evidence_tier = "structural_linkage",
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
    "structural linkage records parser-resolved item anchors only",
    "does not satisfy formal_proof clauses or transfer formal-proof status",
    "does not prove prose-to-code correspondence or whole-program correctness",
  ],
}
