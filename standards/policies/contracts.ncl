# Tiger Style policy registry contracts.
# The registry is authored in Nickel so required fields and closed schemas fail
# before checker code consumes exported JSON.
let NonEmptyString =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.length value > 0
  )
in

let OneOf = fun allowed =>
  std.contract.from_predicate (fun value =>
    std.is_string value && std.array.elem value allowed
  )
in

let ClassificationContract =
  OneOf [
    "hard",
    "advisory",
    "human-process-only",
    "deferred",
  ]
in

let EnforcementLevelContract =
  OneOf [
    "hard",
    "advisory",
    "human-process-only",
    "deferred",
  ]
in

let EnforcementModeContract =
  OneOf [
    "lint",
    "gate",
    "structural",
    "verified",
    "human-oracle",
  ]
in

let MechanismKindContract =
  OneOf [
    "crate-boundary",
    "capability-api",
    "bounded-primitive",
    "type-state",
  ]
in

let SeedRoleContract =
  OneOf [
    "architecture-reference",
    "trajectory-fixture",
    "proof-seed",
    "core-shell-fixture",
  ]
in

let SeverityContract =
  OneOf [
    "error",
    "warning",
    "info",
    "note",
  ]
in

let FixtureStatusContract =
  OneOf [
    "planned",
    "covered",
    "not-applicable",
  ]
in

let CheckerIdContract =
  OneOf [
    "registry",
    "rustc",
    "clippy",
    "rustfmt",
    "tigerstyle",
    "cargo-deny",
    "tracey",
    "verus",
    "tigerstyle-standards",
    "fitness-report",
    "safety-evidence",
    "human-process",
  ]
in

let ReceiptBindingContract =
  OneOf [
    "none",
    "blake3-command-and-source",
  ]
in

let ReceiptProvenanceContract = {
  schema_version | String | default = "",
  runner | String | default = "",
  environment | String | default = "",
  requires_backend | Bool | default = false,
  requires_trust_boundary | Bool | default = false,
}
in

let ReceiptVerifierContract = {
  verifier_id | String | default = "",
  verifier_digest | String | default = "",
  verifier_binary_path | String | default = "",
  verifier_binary_digest | String | default = "",
  verifier_build_closure | String | default = "",
  verifier_build_closure_digest | String | default = "",
  verifier_build_closure_paths | Array NonEmptyString | default = [],
  verifier_nix_derivation | String | default = "",
  verifier_nix_derivation_digest | String | default = "",
  verifier_nix_closure | String | default = "",
  verifier_nix_closure_digest | String | default = "",
  verifier_nix_closure_paths | Array NonEmptyString | default = [],
  verifier_nix_closure_graph_digest | String | default = "",
  verifier_nix_closure_graph_paths | Array NonEmptyString | default = [],
  toolchain_id | String | default = "",
  toolchain_digest | String | default = "",
}
in

let SuppressionRequirements = {
  requires_policy_id | Bool,
  requires_source_section | Bool,
  requires_owner | Bool,
  requires_rationale | Bool,
  requires_review_trigger | Bool,
  requires_safer_default | Bool,
}
in

let FixtureCoverage = {
  status | FixtureStatusContract,
  compliant | Array String,
  noncompliant | Array String,
}
in

let EvidenceContract = {
  command | NonEmptyString,
  expected_signal | NonEmptyString,
}
in

let MechanismContract = {
  kind | MechanismKindContract,
  description | NonEmptyString,
  exported_obligations | Array NonEmptyString,
  crate_boundary | Array NonEmptyString | default = [],
  constructor_authority | Array NonEmptyString | default = [],
  api_obligations | Array NonEmptyString | default = [],
  forbidden_apis | Array NonEmptyString | default = [],
  approved_bounded_apis | Array NonEmptyString | default = [],
  approved_wrapper_scopes | Array NonEmptyString | default = [],
  capacity_witnesses | Array NonEmptyString | default = [],
  backpressure_behaviors | Array NonEmptyString | default = [],
  forbidden_ambient_apis | Array NonEmptyString | default = [],
  core_source_scopes | Array NonEmptyString | default = [],
  shell_constructor_scopes | Array NonEmptyString | default = [],
  approved_capability_names | Array NonEmptyString | default = [],
  type_state_source_scopes | Array NonEmptyString | default = [],
  raw_state_alias_patterns | Array NonEmptyString | default = [],
  raw_transition_patterns | Array NonEmptyString | default = [],
  required_state_markers | Array NonEmptyString | default = [],
  required_transition_methods | Array NonEmptyString | default = [],
}
in

let ProofRailContract = {
  backend | NonEmptyString,
  command | NonEmptyString,
  source_paths | Array NonEmptyString,
  proof_paths | Array NonEmptyString,
  pass_signal | NonEmptyString,
  trust_boundaries | Array NonEmptyString,
  receipt_binding | ReceiptBindingContract | default = "none",
  receipt_provenance | ReceiptProvenanceContract | default = {},
  receipt_verifier | ReceiptVerifierContract | default = {},
}
in

let SeedManifestContract = {
  name | NonEmptyString,
  source_path | NonEmptyString,
  source_commit | NonEmptyString,
  role | SeedRoleContract,
  paths | Array NonEmptyString,
  provenance | NonEmptyString,
  license | NonEmptyString,
  content_hash | NonEmptyString,
  proof_relevant | Bool,
  local_acceptance | Array NonEmptyString,
}
in

let ProofBundleSchemaVersionContract = OneOf [
  "tigerstyle-proof-bundle/v1",
]
in

let ProofBundleEntryContract = {
  policy_id | NonEmptyString,
  rail_index | Number,
  proof_rail | ProofRailContract,
}
in

let ProofBundleArtifactContract = {
  schema_version | ProofBundleSchemaVersionContract,
  generated_from | NonEmptyString,
  registry_schema_version | NonEmptyString,
  proof_bundles | Array ProofBundleEntryContract,
}
in

let mode_rank = fun mode =>
  if mode == "lint" then 0
  else if mode == "gate" then 1
  else if mode == "structural" then 2
  else if mode == "verified" then 3
  else 4
in

let has_mechanism = fun policy kind =>
  std.array.any (fun mechanism => mechanism.kind == kind) policy.mechanisms
in

let policy_cross_fields_valid = fun policy =>
  (mode_rank policy.target_enforcement_mode >= mode_rank policy.current_enforcement_mode)
  && (policy.target_enforcement_mode == policy.current_enforcement_mode || std.string.length policy.escalation_criteria > 0)
  && (policy.target_enforcement_mode != "structural" || std.array.length policy.mechanisms > 0)
  && (policy.target_enforcement_mode != "verified" || std.array.length policy.proof_rails > 0)
  && (policy.default_severity != "error" || std.array.length policy.verification_evidence_requirements > 0)
  && (policy.default_severity != "error" || std.string.length policy.deterministic_checker.command > 0)
in

let seed_cross_fields_valid = fun seed =>
  (!seed.proof_relevant) || std.array.length seed.local_acceptance > 0
in

let RegistryCrossFieldContract =
  std.contract.from_predicate (fun registry =>
    std.array.all policy_cross_fields_valid registry.policies
    && std.array.all seed_cross_fields_valid registry.seed_manifests
  )
in

let PolicyEntryContract = {
  policy_id | NonEmptyString,
  pillar | NonEmptyString,
  source_path | NonEmptyString,
  source_heading | NonEmptyString,
  source_anchor | NonEmptyString,
  summary | NonEmptyString,
  classification | ClassificationContract,
  enforcement_level | EnforcementLevelContract,
  current_enforcement_mode | EnforcementModeContract | default = "lint",
  target_enforcement_mode | EnforcementModeContract | default = "lint",
  escalation_criteria | String | default = "no escalation planned",
  mechanisms | Array MechanismContract | default = [],
  deterministic_checker | EvidenceContract | default = { command = "registry validation", expected_signal = "export succeeds" },
  proof_rails | Array ProofRailContract | default = [],
  checker_owner | NonEmptyString,
  checker_id | CheckerIdContract,
  default_severity | SeverityContract,
  suppression_requirements | SuppressionRequirements,
  fixture_coverage | FixtureCoverage,
  verification_evidence_requirements | Array NonEmptyString,
  classification_rationale | NonEmptyString,
  false_positive_risk | NonEmptyString,
  existing_enforcement | Array NonEmptyString,
  custom_checker_justification | NonEmptyString,
}
in

{
  PolicyRegistry = {
    schema_version | NonEmptyString,
    checker_ids | Array CheckerIdContract,
    policies | Array PolicyEntryContract,
    seed_manifests | Array SeedManifestContract | default = [],
  },

  PolicyEntry = PolicyEntryContract,
  CheckerId = CheckerIdContract,
  Classification = ClassificationContract,
  EnforcementLevel = EnforcementLevelContract,
  Severity = SeverityContract,
  EnforcementMode = EnforcementModeContract,
  Mechanism = MechanismContract,
  ProofRail = ProofRailContract,
  SeedManifest = SeedManifestContract,
  ProofBundleArtifact = ProofBundleArtifactContract,
  ProofBundleSchemaVersion = ProofBundleSchemaVersionContract,
  PolicyRegistryRules = RegistryCrossFieldContract,
}
