//! Rust-owned DTOs for semantic MIR effect-closure profiles and receipts.

use serde::Deserialize;
use serde::Serialize;

pub const PROFILE_SCHEMA_VERSION: &str = "octet-semantic-effect-profile/v1";
pub const RECEIPT_SCHEMA_VERSION: &str = "octet-semantic-effect-receipt/v1";
pub const CLAIM_TIER: &str = "semantic-mir-effect-closure";
pub const COMPILER_STAGE: &str = "rustc-mir-optimized";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MirOperation {
    Alloc,
    Panic,
    Unsafe,
    Clock,
    Random,
    Fs,
    Net,
    Spawn,
    Lock,
    Ffi,
    ExternalMutation,
    InteriorMutation,
    UnwindDrop,
    Unknown,
}

impl MirOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alloc => "alloc",
            Self::Panic => "panic",
            Self::Unsafe => "unsafe",
            Self::Clock => "clock",
            Self::Random => "random",
            Self::Fs => "fs",
            Self::Net => "net",
            Self::Spawn => "spawn",
            Self::Lock => "lock",
            Self::Ffi => "ffi",
            Self::ExternalMutation => "external-mutation",
            Self::InteriorMutation => "interior-mutation",
            Self::UnwindDrop => "unwind-drop",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnwindMode {
    ModeledCleanup,
    RejectUnwind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SupportedSubset {
    pub subset_id: String,
    pub included_constructs: Vec<String>,
    pub excluded_constructs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileBudgets {
    pub max_bodies: usize,
    pub max_blocks_per_body: usize,
    pub max_steps: usize,
    pub max_rounds: usize,
    pub call_edge_limit: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MirClosureProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub claim_tier: String,
    pub compiler_stage: String,
    pub unwind_mode: UnwindMode,
    pub supported_subset: SupportedSubset,
    pub effect_vocabulary: Vec<MirOperation>,
    pub allowed_effects: Vec<MirOperation>,
    pub denied_effects: Vec<MirOperation>,
    pub budgets: ProfileBudgets,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedContractSummary {
    pub summary_id: String,
    pub target_definition: String,
    pub trait_definition: Option<String>,
    pub provider_name: String,
    pub provider_version: String,
    pub cargo_lock_blake3: String,
    pub implementation_blake3: String,
    pub effects: Vec<MirOperation>,
    pub review_label: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileDiagnostic {
    pub code: String,
    pub message: String,
    pub item_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MirNodeResult {
    pub item_id: String,
    pub qualified_path: String,
    pub function_address: String,
    pub direct_effects: Vec<MirOperation>,
    pub closure_effects: Vec<MirOperation>,
    pub trusted_summary_ids: Vec<String>,
    pub unsupported_reasons: Vec<String>,
    pub has_private_local_mutation: bool,
    pub has_external_mutation: bool,
    pub has_interior_mutation: bool,
    pub partial_moves_observed: usize,
    pub ended_borrow_reassignments: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageReport {
    pub is_complete: bool,
    pub expected_function_count: usize,
    pub observed_body_count: usize,
    pub is_budget_exhausted: bool,
    pub missing_functions: Vec<String>,
    pub skipped_bodies: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectIdentities {
    pub unit_id: String,
    pub package_name: String,
    pub target_name: String,
    pub toolchain_blake3: String,
    pub source_blake3: String,
    pub profile_blake3: String,
    pub cargo_lock_blake3: String,
    pub compiler_stage: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MirClosureReceipt {
    pub schema_version: String,
    pub status: crate::effect_closure::AdmissionStatus,
    pub claim_tier: String,
    pub profile_id: String,
    pub unwind_mode: UnwindMode,
    pub subjects: SubjectIdentities,
    pub coverage: CoverageReport,
    pub observed_effects: Vec<MirOperation>,
    pub trusted_contracts: Vec<TrustedContractSummary>,
    pub unsupported_operations: Vec<String>,
    pub nodes: Vec<MirNodeResult>,
    pub diagnostics: Vec<ProfileDiagnostic>,
    pub non_claims: Vec<String>,
    pub receipt_identity: String,
}

pub fn compute_receipt_identity(receipt: &MirClosureReceipt) -> Result<String, String> {
    let mut normalized = receipt.clone();
    normalized.receipt_identity = String::new();
    let bytes =
        serde_json::to_vec(&normalized).map_err(|error| format!("serialize semantic effect receipt: {error}"))?;
    Ok(format!("b3:{}", crate::evidence::artifact_blake3_hex(&bytes)))
}
