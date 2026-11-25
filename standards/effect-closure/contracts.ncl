let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let PositiveInt = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0 && value % 1 == 0
) in
let Blake3Identity = NonEmptyString in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let CatalogSchema = OneOf ["octet-effect-closure-catalog/v1"] in
let WorkspaceSchema = OneOf [
  "octet-effect-closure-workspace-policy/v1",
  "octet-effect-closure-workspace-policy/v2",
] in
let EnvelopePolicySchema = OneOf ["octet-effect-closure-envelope-policy/v1"] in
let BindingProfile = OneOf ["implementation-bound-v1", "lockfile-only-v1"] in
let SummaryJustification = OneOf [
  "checked-proof",
  "reviewed-assumption",
  "unresolved-obligation",
] in
let EffectContract = OneOf ["alloc", "panic", "unsafe", "clock", "random", "fs", "net", "spawn", "lock", "ffi", "unknown"] in
let RoleContract = OneOf ["core", "shell", "adapter", "ffi", "proof"] in
let ProviderKindContract = OneOf ["cargo-package", "toolchain", "workspace"] in
let ModuleRoleContract = {
  path | NonEmptyString,
  role | RoleContract,
} in
let ProviderIdentityContract = {
  kind | ProviderKindContract,
  name | NonEmptyString,
  version | NonEmptyString,
  source | NonEmptyString,
  target | NonEmptyString,
  features | Array NonEmptyString,
} in
let EvidenceReferenceContract = {
  kind | NonEmptyString,
  locator | NonEmptyString,
  blake3 | Blake3Identity,
} in
let GeneratedInputBindingContract = {
  locator | NonEmptyString,
  blake3 | Blake3Identity,
} in
let ImplementationBindingContract = {
  profile | BindingProfile,
  package_envelope_blake3 | Blake3Identity,
  closure_blake3 | Blake3Identity,
  build_context_blake3 | Blake3Identity,
  justification | SummaryJustification,
  generated_inputs | Array GeneratedInputBindingContract | optional,
} in
let CallSummaryContract = {
  call_site_id | Blake3Identity,
  review_label | NonEmptyString,
  callee | NonEmptyString,
  provider | ProviderIdentityContract,
  effects | Array EffectContract,
  evidence | EvidenceReferenceContract,
  implementation_binding | ImplementationBindingContract | optional,
} in
let WorkspacePolicyFields = {
  schema_version | WorkspaceSchema,
  profile_id | OneOf ["deterministic-core"],
  binding_profile | BindingProfile | optional,
  envelope_policy_blake3 | Blake3Identity | optional,
  function_address_baseline | NonEmptyString,
  module_roles | NonEmptyArray | Array ModuleRoleContract,
  call_summaries | Array CallSummaryContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let WorkspacePolicyConsistent = std.contract.from_predicate (fun value =>
  if value.schema_version == "octet-effect-closure-workspace-policy/v2" then
    std.record.has_field "binding_profile" value
    && value.binding_profile == "implementation-bound-v1"
    && std.array.all (fun summary =>
      if summary.provider.kind == "workspace" || summary.provider.kind == "cargo-package" then
        std.record.has_field "implementation_binding" summary
        && summary.implementation_binding.profile == "implementation-bound-v1"
      else
        true
    ) value.call_summaries
  else
    !(std.record.has_field "binding_profile" value)
    || value.binding_profile == "lockfile-only-v1"
) in
let WorkspacePolicyContract = std.contract.Sequence [
  WorkspacePolicyFields,
  WorkspacePolicyConsistent,
] in
let EnvelopePolicyContract = {
  schema_version | EnvelopePolicySchema,
  default_binding_profile | BindingProfile,
  supported_binding_profiles | NonEmptyArray | Array BindingProfile,
  relocation_preserving | Bool,
  included_extensions | NonEmptyArray | Array NonEmptyString,
  included_manifests | NonEmptyArray | Array NonEmptyString,
  build_script | NonEmptyString,
  workspace_build_inputs | NonEmptyArray | Array NonEmptyString,
  excluded_root_dirs | NonEmptyArray | Array NonEmptyString,
  symlinks | OneOf ["reject"],
  unreadable_inputs | OneOf ["incomplete"],
  generated_inputs_policy | OneOf ["static-literal-only-else-incomplete"],
  max_files_per_package | PositiveInt,
  max_bytes_per_package | PositiveInt,
  max_resolve_depth | PositiveInt,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
{
  Effect = EffectContract,
  Role = RoleContract,
  BindingProfile = BindingProfile,
  SummaryJustification = SummaryJustification,
  ProviderIdentity = ProviderIdentityContract,
  GeneratedInputBinding = GeneratedInputBindingContract,
  ImplementationBinding = ImplementationBindingContract,
  CallSummary = CallSummaryContract,
  WorkspacePolicy = WorkspacePolicyContract,
  EnvelopePolicy = EnvelopePolicyContract,
  Catalog = {
    schema_version | CatalogSchema,
    effects | NonEmptyArray | Array EffectContract,
    roles | NonEmptyArray | Array RoleContract,
    role_edges | NonEmptyArray | Array {
      caller | RoleContract,
      callee | RoleContract,
    },
    profiles | NonEmptyArray | Array {
      profile_id | OneOf ["deterministic-core"],
      allowed_effects | Array EffectContract,
      denied_effects | NonEmptyArray | Array EffectContract,
    },
    resolver | NonEmptyString,
    canonicalizer | NonEmptyString,
    claim_scope | OneOf ["selected-source-token-effect-closure"],
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
}
