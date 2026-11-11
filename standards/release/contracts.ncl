# Structure contracts for the Octet release-cohort manifest.
#
# The manifest binds one supported public publication channel to the exact
# source, dependency, toolchain, package, notice, and acceptance-evidence
# identities that qualify it. These contracts reject malformed shapes and
# unknown enum values before exported JSON is consumed.
#
# Cross-field release rules (source/inventory binding, notice completeness,
# imported-content resolution, cache independence, receipt freshness, profile
# support) live in `octet_standards::release_cohort`, which re-checks the
# exported JSON and the checked-in fixtures without network access.
let NonEmptyString = std.contract.from_predicate (
  fun value => std.is_string value && std.string.length (std.string.trim value) > 0
) in
let Enum = fun values =>
  std.contract.from_predicate (fun value => std.is_string value && std.array.elem value values)
in
let Revision = std.contract.from_predicate (
  fun value =>
    std.is_string value
    && std.string.length value == 40
    && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Digest = std.contract.from_predicate (
  fun value => std.is_string value && std.string.is_match "^(b3|sha256):[0-9a-f]{64}$" value
) in
let NarHash = std.contract.from_predicate (
  fun value => std.is_string value && std.string.is_match "^sha256-[A-Za-z0-9+/=]+$" value
) in

let ExclusionRow = {
  path | NonEmptyString,
  reason_id | NonEmptyString,
  detail | NonEmptyString,
} in

let Channel = {
  id | NonEmptyString,
  repository | NonEmptyString,
  flake_ref | NonEmptyString,
  branch | NonEmptyString,
  observed_branch_head | Revision,
  head_observed_at | NonEmptyString,
  head_matches_reviewed_revision | Bool,
  supported | Bool,
  direct_cargo_supported | Bool,
  supported_install_command | NonEmptyString,
  unsupported_selectors | Array NonEmptyString,
} in

let SourceOrigin = {
  repository | NonEmptyString,
  revision | Revision,
  role | NonEmptyString,
} in

let ExportComposition = {
  transform | Array NonEmptyString,
  withheld | Array NonEmptyString,
  rewritten | Array NonEmptyString,
  excluded_paths | Array ExclusionRow,
  required_additions | Array NonEmptyString,
} in

let Lag = {
  intentional | Bool,
  summary | NonEmptyString,
} in

let Source = {
  development | SourceOrigin,
  public | {
    repository | NonEmptyString,
    revision | Revision,
    nar_hash | NarHash,
    export_kind | NonEmptyString,
    derived_from_revision | Revision,
    byte_identical_to_derivation | Bool,
    history | NonEmptyString,
    role | NonEmptyString,
  },
  lag | Lag,
  export_composition | ExportComposition,
} in

let Toolchain = {
  rust_channel | NonEmptyString,
  rust_components | Array NonEmptyString,
  rust_profile | NonEmptyString,
  rust_toolchain_path | NonEmptyString,
  dylint_baseline | Revision,
  dylint_source | NonEmptyString,
  nickel_version | NonEmptyString,
  nickel_revision | Revision,
  nickel_source | NonEmptyString,
  nixpkgs_source | NonEmptyString,
  nixpkgs_revision | Revision,
} in

let Dependency = {
  id | NonEmptyString,
  url | NonEmptyString,
  revision | Revision,
  transport | Enum [ "https", "git-ssh", "nix-flake" ],
  role | NonEmptyString,
  anonymous_at_review | Bool,
  warm_cache_dependency | Bool,
} in

let PackageOutput = {
  output | NonEmptyString,
  cargo_package | NonEmptyString,
  version | NonEmptyString,
  role | NonEmptyString,
} in

let Policies = {
  registry_path | NonEmptyString,
  contracts_path | NonEmptyString,
  self_architecture_policy_path | NonEmptyString,
  export_command | NonEmptyString,
  identities | Array { name | NonEmptyString, path | NonEmptyString },
} in

let CapabilityInventory = {
  path | NonEmptyString,
  source_path | NonEmptyString,
  source_revision | Revision,
  origin_revision | Revision,
  count | Number,
  names | Array NonEmptyString,
  development | {
    path | NonEmptyString,
    revision | Revision,
    count | Number,
  },
  excluded_from_public | Array { name | NonEmptyString, reason_id | NonEmptyString },
  exclusion_reason | NonEmptyString,
} in

let NoticeRow = {
  path | NonEmptyString,
  role | NonEmptyString,
  present_in_public_tree | Bool,
} in

let ImportedMaterial = {
  path | NonEmptyString,
  attribution | NonEmptyString,
  root_license_relied_upon | Bool,
  resolution | Enum [ "excluded", "permission_recorded", "unresolved" ],
  permission_evidence | String,
  included_in_qualified_cohort | Bool,
} in

let Notices = {
  root_license | NonEmptyString,
  required | Array NoticeRow,
  attribution_ids | Array NonEmptyString,
  imported_material | Array ImportedMaterial,
} in

let Receipt = {
  id | NonEmptyString,
  evidence_kind | Enum [ "fresh" ],
  producer | NonEmptyString,
  command | NonEmptyString,
  outcome | Enum [ "success", "blocked" ],
  source_revision | Revision,
  package | NonEmptyString,
  policy | NonEmptyString,
  toolchain | NonEmptyString,
  evidence_digest | Digest,
  detail | String,
} in

let NonEvidence = {
  reference | NonEmptyString,
  evidence_role | Enum [ "non-evidence" ],
  reason | NonEmptyString,
} in

let Acceptance = {
  evidence_digest | Digest,
  evidence_subject | NonEmptyString,
  receipts | Array Receipt,
  non_evidence | Array NonEvidence,
  walkthrough | {
    status | Enum [ "executed", "blocked", "not-run" ],
    command | String,
    detail | NonEmptyString,
  },
  artifact_admission | {
    status | Enum [ "executed", "blocked", "not-run" ],
    command | String,
    detail | NonEmptyString,
  },
} in

let Profile = {
  id | NonEmptyString,
  support_status | Enum [ "verified", "unsupported", "not-claimed" ],
  evidence_refs | Array NonEmptyString,
  detail | NonEmptyString,
} in

let Qualification = {
  status | Enum [ "qualified", "blocked" ],
  blocking_reasons | Array {
    id | NonEmptyString,
    summary | NonEmptyString,
    required_action | NonEmptyString,
  },
} in

{
  ReleaseCohortManifest = {
    schema_version | Enum [ "octet-release-cohort/v1" ],
    cohort_id | NonEmptyString,
    channel | Channel,
    source | Source,
    toolchain | Toolchain,
    dependencies | Array Dependency,
    packages | Array PackageOutput,
    policies | Policies,
    capability_inventory | CapabilityInventory,
    notices | Notices,
    acceptance | Acceptance,
    profiles | Array Profile,
    qualification | Qualification,
    non_claims | Array NonEmptyString,
  },
}
