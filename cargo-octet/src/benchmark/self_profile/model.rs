//! Rust-owned DTOs for compiler self-profile diagnostics and replay verification (#25).

use serde::Deserialize;
use serde::Serialize;

pub const CONFIG_SCHEMA_VERSION: &str = "octet-compiler-self-profile/v1";
pub const REPORT_SCHEMA_VERSION: &str = "octet-self-profile-report/v1";
pub const PHASE_SPAN_SCHEMA_VERSION: &str = "octet-phase-span/v1";
pub const EXPECTED_MEASUREME_FORMAT_VERSION: u32 = 9;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMode {
    PlainCompiler,
    LintCollection,
    ArchitectureCollection,
    SelectedValueFlow,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitBuildStatus {
    Compiled,
    NotCompiled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationSpec {
    pub config_id: ConfigMode,
    pub description: String,
    pub claim_coverage_note: String,
    pub uses_octet_driver: bool,
    pub enables_architecture_collection: bool,
    pub enables_value_flow: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileBudgets {
    pub max_mm_profdata_bytes: usize,
    pub max_events_per_unit: usize,
    pub max_units: usize,
    pub phase_record_budget: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticProfileConfig {
    pub schema_version: String,
    pub profile_id: String,
    pub rustc_format_version: u32,
    pub self_profile_events: String,
    pub configurations: Vec<ConfigurationSpec>,
    pub budgets: ProfileBudgets,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QueryEventSummary {
    pub kind: String,
    pub name: String,
    pub count: u64,
    pub summed_duration_ns: u64,
    pub integer_value_sum: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitProfileObservation {
    pub unit_label: String,
    pub host_or_target_role: String,
    pub cache_state: String,
    pub build_status: UnitBuildStatus,
    pub process_id: u32,
    pub command_line: String,
    pub raw_profdata_path: String,
    pub raw_profdata_blake3: String,
    pub raw_bytes: usize,
    pub compiler_identity: String,
    pub driver_blake3: Option<String>,
    pub lint_lib_blake3: Option<String>,
    pub source_blake3: String,
    pub total_events: usize,
    pub query_events: usize,
    pub activity_events: usize,
    pub cache_hit_events: usize,
    pub artifact_size_events: usize,
    pub distinct_threads: usize,
    pub summed_query_span_ns: u64,
    pub top_queries: Vec<QueryEventSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OctetPhaseSpanRecord {
    pub schema_version: String,
    pub unit_label: String,
    pub process_id: u32,
    pub selected_mir_analysis_ns: u64,
    pub value_flow_bodies_analyzed: usize,
    pub value_flow_steps: usize,
    pub value_flow_rounds: usize,
    pub semantic_bodies_analyzed: usize,
    pub shard_serialization_ns: u64,
    pub shard_bytes_written: usize,
    pub shard_normalization_ns: u64,
    pub policy_evaluation_ns: u64,
    pub unattributed_note: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationComparisonRow {
    pub config_id: ConfigMode,
    pub cache_state: String,
    pub is_instrumented: bool,
    pub wall_elapsed_ns: u64,
    pub policy_exit_code: i32,
    pub policy_findings_count: usize,
    pub claim_coverage_note: String,
    pub units: Vec<UnitProfileObservation>,
    pub octet_phases: Vec<OctetPhaseSpanRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SelfProfileReport {
    pub schema_version: String,
    pub profile_id: String,
    pub workspace_source_blake3: String,
    pub toolchain_identity: String,
    pub profile_config_blake3: String,
    pub baseline_uninstrumented_wall_ns: u64,
    pub instrumented_cold_wall_ns: u64,
    pub profiling_overhead_basis_points: u64,
    pub comparisons: Vec<ConfigurationComparisonRow>,
    pub unattributed_boundaries: Vec<String>,
    pub non_claims: Vec<String>,
    pub report_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayVerificationReport {
    pub schema_version: String,
    pub status: String,
    pub verified_unit_count: usize,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
}

pub fn compute_report_blake3(report: &SelfProfileReport) -> Result<String, String> {
    let mut normalized = report.clone();
    normalized.report_blake3 = String::new();
    let bytes = serde_json::to_vec(&normalized).map_err(|error| format!("serialize self-profile report: {error}"))?;
    Ok(format!("b3:{}", crate::evidence::artifact_blake3_hex(&bytes)))
}
