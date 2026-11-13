use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: String,
    pub profiles: Vec<AnalysisProfile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Architecture,
    Assurance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Compliant,
    Violating,
    Control,
    EmptyScope,
    Disputed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutcome {
    Finding,
    Clean,
    EmptyScope,
    Disputed,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityStatus {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSemantics {
    Recorded,
    Probe,
    Invoke,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserKind {
    None,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationStatus {
    CheckedIn,
    UpstreamArgs,
    NotInstalled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseScope {
    pub selected_sources: Vec<String>,
    pub expected_analyzed_sources: usize,
    pub empty: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusCase {
    pub case_id: String,
    pub task_id: String,
    pub category: Category,
    pub variant: Variant,
    pub source_path: String,
    pub source_identity: String,
    pub scope: CaseScope,
    pub expected: ExpectedOutcome,
    pub rationale: String,
    pub reviewer_role: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisTask {
    pub task_id: String,
    pub category: Category,
    pub description: String,
    pub comparable_subset: String,
    pub required_scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPin {
    pub tool_id: String,
    pub kind: String,
    pub executable: String,
    pub version_pin: String,
    pub source_pin: String,
    pub config_path: Vec<String>,
    pub config_identity: Vec<String>,
    pub configuration_status: ConfigurationStatus,
    pub rules_pin: String,
    pub probe_argv: Vec<String>,
    pub run_argv: Vec<String>,
    pub execution: ExecutionSemantics,
    pub parser: ParserKind,
    pub setup_source: String,
    pub setup_steps: Vec<String>,
    pub required_config_files: Vec<String>,
    pub claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applicability {
    pub task_id: String,
    pub tool_id: String,
    pub status: ApplicabilityStatus,
    pub case_ids: Vec<String>,
    pub comparable_subset: String,
    pub scope: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionTarget {
    pub selection_id: String,
    pub source_repository: String,
    pub source_revision: String,
    pub workspace_path: String,
    pub immutable: bool,
    pub notes: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionToolPlan {
    pub tool_id: String,
    pub setup_steps: Vec<String>,
    pub required_config_files: Vec<String>,
    pub initial_command: Vec<String>,
    pub incremental_command: Vec<String>,
    pub unavailable_metrics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionPlan {
    pub clean_consumer: AdoptionTarget,
    pub real_consumer: AdoptionTarget,
    pub tools: Vec<AdoptionToolPlan>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SamplePlan {
    pub warm_state: super::WarmState,
    pub cache_state: super::CacheState,
    pub target: String,
    pub sample_count: usize,
    pub timeout_ms: u64,
    pub environment_facts: Vec<String>,
    pub dependency_differences: Vec<String>,
    pub unavailable_metrics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeControlDeclaration {
    pub control_id: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageBoundary {
    pub corpus_id: String,
    pub bounded: bool,
    pub complete_rust_coverage: bool,
    pub statement: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisProfile {
    pub profile_id: String,
    pub source_revision: String,
    pub rust_toolchain: String,
    pub evidence_tier: String,
    pub declared_before_measurement: bool,
    pub corpus: Vec<CorpusCase>,
    pub tasks: Vec<AnalysisTask>,
    pub tools: Vec<ToolPin>,
    pub applicability: Vec<Applicability>,
    pub adoption: AdoptionPlan,
    pub sample_plan: SamplePlan,
    pub negative_controls: Vec<NegativeControlDeclaration>,
    pub coverage_boundary: CoverageBoundary,
    pub non_claims: Vec<String>,
}
