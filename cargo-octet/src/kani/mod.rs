mod adapter;
mod harness;
mod probe;
mod review;
mod shell;
#[cfg(test)]
mod tests;

pub const PROFILE_SCHEMA: &str = "octet-kani-producer-profile/v1";
pub const PRODUCER_PROFILE_ID: &str = "kani-0.68.0-panic-freedom";
pub const EXPECTED_PROFILE_IDENTITY: &str = "b3:2194d49b606a2a75ec957fc270c9a3b3b2f100e38f7f6f683458aaf12951a378";
pub const OBSERVATION_SCHEMA: &str = "octet-kani-producer-observation/v1";
pub const ASSUMPTION_REVIEW_SCHEMA: &str = "octet-kani-assumption-review/v1";
pub const EXPORT_SCHEMA: &str = "kani-export/v1";
pub const RECEIPT_SCHEMA: &str = "octet-kani-producer-receipt/v1";
pub const KANI_SUMMARY_SCHEMA: &str = "cargo-kani-summary/v1";
pub const DETAILS_ARTIFACT: &str = "kani-producer-details.json";
pub const RAW_EXPORT_ARTIFACT: &str = "kani-native-export.json";
pub const NORMALIZED_SUMMARY_ARTIFACT: &str = "kani-normalized-summary.json";
pub const EVIDENCE_TIER: &str = "transcript_assertion";
pub const REQUIRED_NON_CLAIM: &str = "Kani checks bounded symbolic execution over declared harnesses and unwind bounds only; it does not prove whole-program correctness, verifier soundness, or release eligibility";

const PROFILE_JSON: &str = include_str!("../../../standards/kani-producer/profile.json");
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HarnessSpec {
    pub name: String,
    pub kind: String,
    pub unwind: u64,
    pub cover_properties: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Pin {
    pub kani_version: String,
    pub provider_revision: String,
    pub closure_identity: String,
    pub rustc_release: String,
    pub cbmc_version: String,
    pub solver_mode: String,
    pub solver_identity: String,
    pub export_flags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub max_export_bytes: u64,
    pub max_source_files: usize,
    pub poll_interval_ms: u64,
    pub teardown_timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub schema_version: String,
    #[serde(rename = "profile_id")]
    pub id: String,
    pub evidence_tier: String,
    pub package: String,
    pub target: String,
    pub cargo_features: Vec<String>,
    pub cargo_cfg: Vec<String>,
    pub kani_pin: Pin,
    pub command_prefix: Vec<String>,
    pub harnesses: Vec<HarnessSpec>,
    pub default_unwind: u64,
    #[serde(rename = "require_unwind_assertions")]
    pub is_require_unwind_assertions: bool,
    #[serde(rename = "require_reachable_cover")]
    pub is_require_reachable_cover: bool,
    #[serde(rename = "require_assumption_review")]
    pub is_require_assumption_review: bool,
    pub limits: Limits,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolchainObservation {
    pub kani_version: String,
    pub provider_revision: String,
    pub closure_identity: String,
    pub rustc_release: String,
    pub rustc_commit_hash: String,
    pub cbmc_version: String,
    pub solver_mode: String,
    pub solver_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssumptionReviewRecord {
    pub schema_version: String,
    pub source_manifest_identity: String,
    pub harness_names: Vec<String>,
    #[serde(rename = "assumptions_reviewed")]
    pub is_assumptions_reviewed: bool,
    pub assumption_status: String,
    pub reviewer: String,
    pub notes: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Observation {
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
    pub toolchain: ToolchainObservation,
    pub exit_code: i32,
    #[serde(rename = "timed_out", skip_serializing_if = "Option::is_none")]
    pub timed_out: Option<bool>,
    #[serde(rename = "truncated", skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    pub bounded_log_identity: String,
    pub raw_export_json: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumption_review: Option<AssumptionReviewRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CheckLocation {
    pub file: String,
    pub function: String,
    pub line: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PropertyTotals {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
    pub undetermined: u64,
    pub unreachable: u64,
    pub cover_satisfied: u64,
    pub cover_unsatisfied: u64,
    pub unwind_passed: u64,
    pub unwind_failed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HarnessNormalizedResult {
    pub package: String,
    pub target: String,
    pub name: String,
    pub verification_status: String,
    pub unwind_bound: u64,
    pub totals: PropertyTotals,
    pub check_locations: Vec<CheckLocation>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub package: String,
    pub target: String,
    pub command_argv: Vec<String>,
    pub source_manifest_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile_identity: Option<String>,
    pub toolchain: ToolchainObservation,
    pub selected_harnesses: Vec<String>,
    pub executed_harnesses: Vec<String>,
    pub harnesses: Vec<HarnessNormalizedResult>,
    pub property_totals: PropertyTotals,
    pub execution_status: String,
    pub assumption_status: String,
    #[serde(rename = "assumptions_reviewed")]
    pub is_assumptions_reviewed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumption_review_identity: Option<String>,
    pub raw_export_identity: String,
    pub normalized_summary_identity: String,
    pub bounded_log_identity: String,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub normalized_summary_json: Option<String>,
    pub raw_export_json: Option<String>,
    pub diagnostics: Vec<String>,
    pub status: String,
    pub finding_messages: Vec<(String, String, String)>,
}

pub fn evaluate_observation(text: &str) -> Evaluation {
    adapter::check_observation(text)
}

pub fn run(
    workspace_root: &std::path::Path,
    profile: &Profile,
    assumption_review_path: Option<&std::path::Path>,
    export_dir_override: Option<&std::path::Path>,
    process_env: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Result<(String, i32), String> {
    shell::run(workspace_root, profile, assumption_review_path, export_dir_override, process_env)
}

pub fn is_producer_profile(profile: Option<&str>) -> bool {
    profile == Some(PRODUCER_PROFILE_ID)
}

pub fn selected_profile(profile_id: &str) -> Result<Profile, String> {
    if prefixed_identity(PROFILE_JSON.as_bytes()) != EXPECTED_PROFILE_IDENTITY {
        return Err("embedded Kani producer profile registry identity mismatch".to_owned());
    }
    let profile: Profile =
        serde_json::from_str(PROFILE_JSON).map_err(|error| format!("parse embedded Kani producer profile: {error}"))?;
    if profile.schema_version != PROFILE_SCHEMA {
        return Err(format!("unsupported Kani producer profile schema `{}`", profile.schema_version));
    }
    if profile.id != profile_id {
        return Err(format!("unknown Kani producer profile `{profile_id}`"));
    }
    Ok(profile)
}

pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(4);
    if details.schema_version != RECEIPT_SCHEMA {
        diagnostics.push(format!("unsupported Kani producer receipt schema `{}`", details.schema_version));
    }
    if details.profile_identity != EXPECTED_PROFILE_IDENTITY {
        diagnostics.push("Kani producer receipt profile_identity is stale".to_owned());
    }
    if details.evidence_tier != EVIDENCE_TIER {
        diagnostics.push(format!("Kani producer evidence_tier must be `{EVIDENCE_TIER}`"));
    }
    if !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("Kani producer receipt missing required bounded non-claim".to_owned());
    }
    if !is_b3_identity(&details.source_manifest_identity)
        || !is_b3_identity(&details.raw_export_identity)
        || !is_b3_identity(&details.normalized_summary_identity)
        || !is_b3_identity(&details.bounded_log_identity)
    {
        diagnostics.push("Kani producer receipt missing required BLAKE3 artifact identity".to_owned());
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
