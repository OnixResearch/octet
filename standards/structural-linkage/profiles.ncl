let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Profile = {
  profile_id | NonEmptyString,
  producer | NonEmptyString,
  expected_report_schema | NonEmptyString,
  expected_anchor_kind | std.contract.Equal "rust_item_v1",
  expected_adapter | NonEmptyString,
  expected_parser | NonEmptyString,
  expected_structural_schema | NonEmptyString,
  mapping_id | NonEmptyString,
  mapping_version | NonEmptyString,
  namespace_prefix | NonEmptyString,
  evidence_class | std.contract.Equal "structural_linkage",
  evidence_tier | std.contract.Equal "structural_linkage",
  supported_relations | Array NonEmptyString,
  command_execution_allowed | std.contract.Equal false,
  non_claims | Array NonEmptyString,
} in
let ProfileRegistry = {
  schema_version | std.contract.Equal "octet-structural-linkage-profiles/v1",
  profiles | Array Profile,
} in
let shared_non_claims = [
  "structural linkage records parser-resolved item anchors only",
  "does not satisfy formal_proof clauses or transfer formal-proof status",
  "does not prove prose-to-code correspondence, callee behavior, or test adequacy",
] in
{
  schema_version = "octet-structural-linkage-profiles/v1",
  profiles = [
    {
      profile_id = "cairn-structural-trace-v2",
      producer = "cairn",
      expected_report_schema = "cairn.content-bound-trace-index.v2",
      expected_anchor_kind = "rust_item_v1",
      expected_adapter = "cairn.saved-rust-bytes.v1",
      expected_parser = "syn-2.0.117",
      expected_structural_schema = "cairn.rust-item-binding.v1",
      mapping_id = "octet-cairn-structural-linkage-v1",
      mapping_version = "1",
      namespace_prefix = "cairn",
      evidence_class = "structural_linkage",
      evidence_tier = "structural_linkage",
      supported_relations = ["impl", "verify"],
      command_execution_allowed = false,
      non_claims = shared_non_claims,
    },
    {
      profile_id = "buoy-comparison-pilot-v1",
      producer = "buoy-comparison-pilot",
      expected_report_schema = "cairn.buoy-comparison.receipt.v1",
      expected_anchor_kind = "rust_item_v1",
      expected_adapter = "cairn.buoy-comparison-pilot.v1",
      expected_parser = "buoy-01096fb",
      expected_structural_schema = "cairn.buoy-comparison.receipt.v1",
      mapping_id = "octet-buoy-comparison-pilot-v1",
      mapping_version = "1",
      namespace_prefix = "buoy",
      evidence_class = "structural_linkage",
      evidence_tier = "structural_linkage",
      supported_relations = ["impl", "verify"],
      command_execution_allowed = false,
      non_claims = shared_non_claims,
    },
  ],
} | ProfileRegistry
