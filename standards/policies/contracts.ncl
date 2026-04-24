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

let PolicyEntryContract = {
  policy_id | NonEmptyString,
  pillar | NonEmptyString,
  source_path | NonEmptyString,
  source_heading | NonEmptyString,
  source_anchor | NonEmptyString,
  summary | NonEmptyString,
  classification | ClassificationContract,
  enforcement_level | EnforcementLevelContract,
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
  },

  PolicyEntry = PolicyEntryContract,
  CheckerId = CheckerIdContract,
  Classification = ClassificationContract,
  EnforcementLevel = EnforcementLevelContract,
  Severity = SeverityContract,
}
