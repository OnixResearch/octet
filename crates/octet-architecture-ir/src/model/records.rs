#[derive(
    Clone,
    Copy,
    Debug,
    serde::Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize
)]
#[serde(rename_all = "kebab-case")]
pub enum UnknownFactKind {
    AmbiguousTraitDispatch,
    DynamicDispatch,
    Ffi,
    UnsupportedExpansion,
    OpaqueGeneratedCode,
    MissingBody,
    UnresolvedDefinition,
    UnsupportedCompilerShape,
    AmbiguousValueProvenance,
    MissingValueProvenance,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownCompilerFact {
    pub unknown_id: String,
    pub kind: UnknownFactKind,
    pub owner_item_id: Option<String>,
    pub target_identity: Option<String>,
    pub span_id: Option<String>,
    pub detail: String,
}

#[derive(
    Clone,
    Copy,
    Debug,
    serde::Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize
)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionStatus {
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureShard {
    pub schema_version: String,
    pub shard_id: String,
    pub unit: super::CompilationUnit,
    pub collection_status: CollectionStatus,
    pub spans: Vec<super::SourceSpan>,
    pub facts: Vec<super::CompilerArchitectureFact>,
    pub unknowns: Vec<UnknownCompilerFact>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedCompilationUnit {
    pub unit: super::CompilationUnit,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRoleScope {
    pub source_scope: String,
    pub role: super::CompilerArchitectureRole,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorCompilationUnit {
    pub rustc_crate_name: String,
    pub manifest_dir: String,
    pub source_path: String,
    pub test_configuration: bool,
    pub source_roles: Vec<SourceRoleScope>,
    pub unit: super::CompilationUnit,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureCollectorContext {
    pub schema_version: String,
    pub workspace_root: String,
    pub output_directory: String,
    pub limits: super::ArchitectureCollectionLimits,
    pub selected_units: Vec<CollectorCompilationUnit>,
    #[serde(default = "super::empty_value_flow_selections")]
    pub value_flow_selections: Vec<super::CompilerValueFlowSelection>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureIr {
    pub schema_version: String,
    pub architecture_ir_id: String,
    pub policy_blake3: String,
    pub units: Vec<super::CompilationUnit>,
    pub unit_memberships: Vec<UnitFactMembership>,
    pub spans: Vec<super::SourceSpan>,
    pub facts: Vec<super::CompilerArchitectureFact>,
    pub unknowns: Vec<UnknownCompilerFact>,
    pub non_claims: Vec<String>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    serde::Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize
)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageStatus {
    Complete,
    Incomplete,
}

/// Observations from one validated shard, retained before cross-unit deduplication.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitFactMembership {
    pub unit_id: String,
    pub fact_ids: Vec<String>,
    pub unknown_ids: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureCoverage {
    pub schema_version: String,
    pub coverage_id: String,
    pub status: CoverageStatus,
    pub expected_unit_ids: Vec<String>,
    pub observed_unit_ids: Vec<String>,
    pub missing_unit_ids: Vec<String>,
    pub unexpected_unit_ids: Vec<String>,
    pub duplicate_unit_ids: Vec<String>,
    pub invalid_shard_ids: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureIssue {
    pub schema_version: String,
    pub issue_id: String,
    pub code: String,
    pub unit_id: Option<String>,
    pub fact_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureReceipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub status: CollectionStatus,
    pub policy_blake3: String,
    pub coverage_id: String,
    pub architecture_ir_id: Option<String>,
    pub issue_ids: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidCompilerArchitectureShard {
    pub shard_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationInput {
    pub policy_blake3: String,
    pub limits: super::ArchitectureCollectionLimits,
    pub expected_units: Vec<ExpectedCompilationUnit>,
    pub shards: Vec<CompilerArchitectureShard>,
    pub invalid_shards: Vec<InvalidCompilerArchitectureShard>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationOutcome {
    pub architecture_ir: Option<CompilerArchitectureIr>,
    pub coverage: CompilerArchitectureCoverage,
    pub issues: Vec<CompilerArchitectureIssue>,
    pub receipt: CompilerArchitectureReceipt,
}
