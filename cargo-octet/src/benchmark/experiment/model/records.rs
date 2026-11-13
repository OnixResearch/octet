use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarmState {
    Cold,
    Warm,
    NoChange,
    Incremental,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheState {
    Cold,
    Warm,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    Completed,
    TimedOut,
    Failed,
    Unavailable,
    Unsupported,
    NotRun,
    Unparsed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompileStatus {
    Compiled,
    NotCompiled,
    NotApplicable,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementOrigin {
    Local,
    RecordedFixture,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Local,
    Recorded,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationEnvironment {
    pub target: String,
    pub host_os: String,
    pub host_arch: String,
    pub rust_toolchain: String,
    pub cache_state: CacheState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationScope {
    pub selected_sources: Vec<String>,
    pub analyzed_sources: Vec<String>,
    pub scope_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedFinding {
    pub case_id: String,
    pub rule_id: String,
    pub path: String,
    pub line: u32,
    pub message: String,
    pub suppressed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedSuppression {
    pub case_id: String,
    pub rule_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolObservation {
    pub observation_id: String,
    pub tool_id: String,
    pub task_id: String,
    pub case_id: Option<String>,
    pub source_identity: String,
    pub tool_version: String,
    pub origin: MeasurementOrigin,
    pub environment: ObservationEnvironment,
    pub scope: ObservationScope,
    pub status: ObservationStatus,
    pub compile_status: CompileStatus,
    pub findings: Vec<ObservedFinding>,
    pub suppressions: Vec<ObservedSuppression>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_identity: String,
    pub stderr_identity: String,
    pub exit_code: Option<i32>,
    pub wall_duration_ns: Option<u64>,
    pub warm_state: WarmState,
    pub sample_index: u32,
    pub failure_reason: Option<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAvailability {
    pub tool_id: String,
    pub status: Availability,
    pub version_output: String,
    pub version_identity: String,
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdoptionPhase {
    Initial,
    Incremental,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionObservation {
    pub observation_id: String,
    pub tool_id: String,
    pub target_selection: String,
    pub phase: AdoptionPhase,
    pub origin: MeasurementOrigin,
    pub measured: bool,
    pub wall_duration_ns: Option<u64>,
    pub warm_state: WarmState,
    pub environment: ObservationEnvironment,
    pub stdout: String,
    pub stderr: String,
    pub stdout_identity: String,
    pub stderr_identity: String,
    pub exit_code: Option<i32>,
    pub unavailable_reason: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationSet {
    pub schema_version: String,
    pub profile_id: String,
    pub tool_availability: Vec<ToolAvailability>,
    pub observations: Vec<ToolObservation>,
    pub adoption: Vec<AdoptionObservation>,
}
