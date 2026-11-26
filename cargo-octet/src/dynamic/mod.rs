mod evaluate;
mod manifest;
mod probe;
mod shell;
#[cfg(test)]
mod tests;

pub const PROFILE_SCHEMA: &str = "octet-dynamic-qualified-profiles/v1";
pub const MIRI_QUALIFIED_PROFILE_ID: &str = "miri-unsafe-boundary-qualified";
pub const LOOM_QUALIFIED_PROFILE_ID: &str = "loom-concurrency-qualified";
pub const EXPECTED_PROFILE_IDENTITY: &str = "b3:7aa345b6df1e329bf43aa6dbb4799e9dd07629caaca3b2827f07707b9ea2cf84";
pub const QUALIFICATION_INPUT_SCHEMA: &str = "octet-dynamic-qualification-input/v1";
pub const MATRIX_RECEIPT_SCHEMA: &str = "compiler-lab-dynamic-matrix/v1";
pub const RECEIPT_SCHEMA: &str = "octet-dynamic-qualification-receipt/v1";
pub const DETAILS_ARTIFACT: &str = "dynamic-qualification-details.json";
pub const EVIDENCE_TIER: &str = "transcript_assertion";
pub const REQUIRED_NON_CLAIM: &str = "Qualified dynamic evidence covers declared tests, schedules, seeds, and target configurations only; it does not prove whole-program correctness, unexecuted paths, or release eligibility.";

const PROFILE_JSON: &str = include_str!("../../../standards/dynamic-evidence/qualified-profiles.json");
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MatrixCellSpec {
    pub cell_id: String,
    pub tool: String,
    pub target_triple: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub borrow_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preemption_bound: Option<u64>,
    #[serde(rename = "required")]
    pub is_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MiriSpec {
    pub toolchain_pin: String,
    pub target_triple: String,
    pub borrow_model: String,
    #[serde(rename = "check_provenance")]
    pub is_check_provenance: bool,
    pub isolation: String,
    pub seeds: Vec<u64>,
    pub miri_flags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoomSpec {
    pub model_identity: String,
    pub max_threads: u64,
    pub max_branches: u64,
    pub max_permutations: u64,
    pub preemption_bound: u64,
    pub stopping_rules: String,
    pub expected_model_entry_marker: String,
    pub expected_model_completion_marker: String,
    pub rustflags: String,
    pub encoded_rustflags: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub max_source_files: usize,
    pub max_input_bytes: u64,
    pub poll_interval_ms: u64,
    pub teardown_timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    #[serde(rename = "profile_id")]
    pub id: String,
    pub tool: String,
    pub evidence_tier: String,
    pub package: String,
    pub target: String,
    pub cargo_features: Vec<String>,
    pub cargo_cfg: Vec<String>,
    pub command_argv: Vec<String>,
    pub test_inventory: Vec<String>,
    pub expected_workload: String,
    pub limits: Limits,
    pub matrix_cells: Vec<MatrixCellSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miri: Option<MiriSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loom: Option<LoomSpec>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
struct Registry {
    schema_version: String,
    profiles: Vec<Profile>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompilerProbe {
    pub cargo_version: String,
    pub rustc_release: String,
    pub commit_hash: String,
    pub host_triple: String,
    pub tool_kind: String,
    pub tool_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysroot_identity: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CellObservation {
    pub cell_id: String,
    pub disposition: String,
    pub target_triple: String,
    pub tool_version: String,
    pub rustc_release: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miri_flags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rustflags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_rustflags: Option<String>,
    pub tests_started: u64,
    pub tests_passed: u64,
    pub tests_failed: u64,
    pub tests_ignored: u64,
    pub tests_filtered_out: u64,
    pub executed_tests: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<String>>,
    #[serde(rename = "model_entered", skip_serializing_if = "Option::is_none")]
    pub model_entered: Option<bool>,
    #[serde(rename = "model_completed", skip_serializing_if = "Option::is_none")]
    pub model_completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permutations_explored: Option<u64>,
    pub exit_code: i32,
    #[serde(rename = "timed_out", skip_serializing_if = "Option::is_none")]
    pub timed_out: Option<bool>,
    #[serde(rename = "truncated", skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    pub bounded_log_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_messages: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Input {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub source_manifest_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_source_manifest_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_lockfile_identity: Option<String>,
    pub command_argv: Vec<String>,
    pub compiler_probe: CompilerProbe,
    pub observed_runs: Vec<CellObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MatrixCellSummary {
    pub cell_id: String,
    pub target_triple: String,
    pub status: String,
    pub tests_started: u64,
    pub tests_passed: u64,
    pub tests_failed: u64,
    pub tests_ignored: u64,
    pub permutations_explored: u64,
    pub bounded_log_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub tool: String,
    pub package: String,
    pub target: String,
    pub command_argv: Vec<String>,
    pub source_manifest_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile_identity: Option<String>,
    pub compiler_probe: CompilerProbe,
    pub qualification_status: String,
    pub matrix_cells: Vec<MatrixCellSummary>,
    pub tests_started: u64,
    pub tests_passed: u64,
    pub tests_failed: u64,
    pub tests_ignored: u64,
    pub tests_filtered_out: u64,
    pub executed_tests: Vec<String>,
    pub filters: Vec<String>,
    #[serde(rename = "model_entered")]
    pub is_model_entered: bool,
    #[serde(rename = "model_completed")]
    pub is_model_completed: bool,
    pub permutations_explored: u64,
    pub bounded_log_identity: String,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub diagnostics: Vec<String>,
    pub status: String,
    pub findings: Vec<String>,
}

pub fn evaluate_input(text: &str, requested_profile: Option<&str>) -> Evaluation {
    evaluate::run_input(text, requested_profile)
}

pub fn run(
    workspace_root: &std::path::Path,
    profile: &Profile,
    process_env: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Result<(String, i32), String> {
    shell::run(workspace_root, profile, process_env)
}

pub fn is_qualified_profile(profile: Option<&str>) -> bool {
    matches!(profile, Some(MIRI_QUALIFIED_PROFILE_ID | LOOM_QUALIFIED_PROFILE_ID))
}

pub fn selected_profile(profile_id: &str) -> Result<Profile, String> {
    if prefixed_identity(PROFILE_JSON.as_bytes()) != EXPECTED_PROFILE_IDENTITY {
        return Err("embedded dynamic qualified profiles registry identity mismatch".to_owned());
    }
    let registry: Registry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded dynamic qualified profiles: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err(format!("unsupported dynamic qualified profile schema `{}`", registry.schema_version));
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| format!("unknown qualified dynamic profile `{profile_id}`"))
}

pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(4);
    if details.schema_version != RECEIPT_SCHEMA {
        diagnostics.push(format!("unsupported dynamic qualification receipt schema `{}`", details.schema_version));
    }
    if details.profile_identity != EXPECTED_PROFILE_IDENTITY {
        diagnostics.push("dynamic qualification receipt profile_identity is stale".to_owned());
    }
    if details.evidence_tier != EVIDENCE_TIER {
        diagnostics.push(format!("dynamic qualification evidence_tier must be `{EVIDENCE_TIER}`"));
    }
    if !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("dynamic qualification receipt missing required non-claim".to_owned());
    }
    if !is_b3_identity(&details.source_manifest_identity) || !is_b3_identity(&details.bounded_log_identity) {
        diagnostics.push("dynamic qualification receipt missing BLAKE3 source or log identity".to_owned());
    }
    diagnostics
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

pub fn is_b3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(|hex| {
        hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    })
}
