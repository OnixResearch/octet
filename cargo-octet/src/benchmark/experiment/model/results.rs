use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolTaskMetrics {
    pub category: super::Category,
    pub task_id: String,
    pub tool_id: String,
    pub applicability_status: super::ApplicabilityStatus,
    pub comparable_subset: String,
    pub availability: super::Availability,
    pub comparable_cases: u64,
    pub true_positive: u64,
    pub false_positive: u64,
    pub false_negative: u64,
    pub true_negative: u64,
    pub unsupported_cases: u64,
    pub inconclusive_cases: u64,
    pub incomparable_cases: u64,
    pub suppressed_cases: u64,
    pub empty_scope_cases: u64,
    pub disputed_cases: u64,
    pub missing_execution_cases: u64,
    pub precision_basis_points: Option<u64>,
    pub recall_basis_points: Option<u64>,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionSummary {
    pub tool_id: String,
    pub availability: super::Availability,
    pub setup_steps: u64,
    pub config_files: u64,
    pub clean_initial_ns: Option<u64>,
    pub clean_incremental_ns: Option<u64>,
    pub real_initial_ns: Option<u64>,
    pub real_incremental_ns: Option<u64>,
    pub incremental_over_initial_basis_points: Option<u64>,
    pub measured_targets: Vec<String>,
    pub unavailable_metrics: Vec<String>,
    pub unavailable_reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceSummary {
    pub executed_locally: Vec<String>,
    pub recorded_fixtures: Vec<String>,
    pub unavailable: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedProjection {
    pub architecture: Vec<ToolTaskMetrics>,
    pub assurance: Vec<ToolTaskMetrics>,
    pub adoption: Vec<AdoptionSummary>,
    pub provenance: ProvenanceSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlOutcome {
    Contained,
    Escaped,
    NotRun,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeControlRecord {
    pub control_id: String,
    pub outcome: ControlOutcome,
    pub evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema_version: String,
    pub profile_identity: String,
    pub profile: super::AnalysisProfile,
    pub tool_availability: Vec<super::ToolAvailability>,
    pub observations_identity: String,
    pub observations: Vec<super::ToolObservation>,
    pub adoption_identity: String,
    pub adoption: Vec<super::AdoptionObservation>,
    pub derived: DerivedProjection,
    pub derived_identity: String,
    pub controls: Vec<NegativeControlRecord>,
    pub controls_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub artifact: Option<Artifact>,
    pub diagnostics: Vec<String>,
}
