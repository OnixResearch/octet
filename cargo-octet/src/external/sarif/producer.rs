// r[impl octet.external_lint.sarif_producer_execution]
#![allow(clippy::assigning_clones)]

use serde::Deserialize;
use serde::Serialize;

/// Schema of the checked Nickel producer profile registry.
pub(crate) const PROFILE_SCHEMA: &str = "octet-sarif-producer-profiles/v1";
/// BLAKE3 identity of the checked `standards/sarif/producers.json` bytes.
pub(crate) const PROFILE_IDENTITY_VALUE: &str = "b3:bc12c39f5fdb66607f0acd91f00cd72785f75f526a4f5504d4e2e847492b94cd";
/// Versioned schema of one Octet-owned producer execution receipt.
pub(crate) const EXECUTION_SCHEMA: &str = "octet-sarif-producer-execution/v1";
/// Artifact that carries the execution receipt next to the raw report and projection.
pub(crate) const EXECUTION_ARTIFACT: &str = "sarif-producer-execution.json";
/// Artifact directory prefix, relative to the workspace root, that holds producer staging state.
pub(crate) const STAGE_ROOT: &str = ".octet/sarif-producer";
const REPORT_FILE_NAME: &str = "report.sarif";
const DATABASE_DIR_NAME: &str = "codeql-database";
const DATABASE_BINDING_FILE_NAME: &str = "octet-sarif-database-binding.json";
const EVIDENCE_ORIGIN_EXECUTED: &str = "octet_executed";
pub(crate) const OUTCOME_COMPLETED: &str = "completed";
pub(crate) const OUTCOME_FAILED: &str = "failed";
pub(crate) const OUTCOME_UNSUPPORTED: &str = "unsupported";
pub(crate) const OUTCOME_TOOL_ERROR: &str = "tool_error";
pub(crate) const OUTCOME_TIMEOUT: &str = "timeout";
pub(crate) const OUTCOME_INCOMPLETE: &str = "incomplete_analysis";
const STATUS_SUCCEEDED: &str = "succeeded";
const STATUS_FAILED: &str = "failed";
const STATUS_REUSED: &str = "reused";
const PRODUCER_SEMGREP: &str = "semgrep";
const PRODUCER_CODEQL: &str = "codeql";
const LOCAL_RULE_FILE: &str = "local-rule-file";
const QUERY_PACK_SET: &str = "query-pack-set";
const IDENTITY_SOURCE_CHECKED_BYTES: &str = "checked-file-bytes";
const IDENTITY_SOURCE_DECLARED_PIN: &str = "declared-pin";
const DATABASE_POLICY_REUSE_OR_CREATE: &str = "reuse-or-create";
const BUILD_MODE_SOURCE_ONLY: &str = "source-only";
const BUILD_MODE_NONE: &str = "none";
const SNAPSHOT_IDENTITY_DOMAIN: &[u8] = b"octet-sarif-source-snapshot-v1\0";
const QUERY_PACK_IDENTITY_DOMAIN: &[u8] = b"octet-sarif-codeql-query-pack-v1\0";
const QUERY_PACK_SET_IDENTITY_DOMAIN: &[u8] = b"octet-sarif-codeql-pack-set-v1\0";
pub(crate) const MAX_EXECUTABLE_BYTES: u64 = 512 * 1_024 * 1_024;
const EXIT_SUCCESS: i32 = 0;

const PROFILE_JSON: &str = include_str!("../../../../standards/sarif/producers.json");

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Registry {
    schema_version: String,
    profiles: Vec<Profile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Executable {
    pub name: String,
    pub declared_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RuleSource {
    pub kind: String,
    pub path: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryPack {
    pub name: String,
    pub version: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Database {
    pub language: String,
    pub build_mode: String,
    pub policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Scope {
    pub paths: Vec<String>,
    pub extensions: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub max_files: usize,
    pub max_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RustScope {
    pub build_mode: String,
    pub target: String,
    pub features: Vec<String>,
    pub packages: Vec<String>,
    pub unsupported_features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Limits {
    pub wall_clock_ms: u64,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub max_report_bytes: u64,
    pub max_memory_mb: u64,
    pub max_threads: u64,
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profile {
    pub profile_id: String,
    pub producer: String,
    pub intake_profile_id: String,
    pub driver_name: String,
    pub executable: Executable,
    pub supported_languages: Vec<String>,
    pub rules: RuleSource,
    pub query_packs: Vec<QueryPack>,
    pub database: Option<Database>,
    pub scope: Scope,
    pub rust: RustScope,
    pub limits: Limits,
    pub non_claims: Vec<String>,
}

/// One selected source file and its exact content identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SourceFile {
    pub path: String,
    pub identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ScopeReceipt {
    pub declared_paths: Vec<String>,
    pub selected_paths: Vec<String>,
    pub missing_paths: Vec<String>,
    pub extensions: Vec<String>,
    pub exclude_globs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RustScopeReceipt {
    pub build_mode: String,
    pub target: String,
    pub features: Vec<String>,
    pub packages: Vec<String>,
    pub unsupported_features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LimitReceipt {
    pub wall_clock_ms: u64,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub max_report_bytes: u64,
    pub max_memory_mb: u64,
    pub max_threads: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ExecutableReceipt {
    pub name: String,
    pub declared_version: String,
    pub observed_version: String,
    pub resolved_path: String,
    pub identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RuleReceipt {
    pub kind: String,
    pub path: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryPackReceipt {
    pub name: String,
    pub version: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct StepReceipt {
    pub step: String,
    pub argv: Vec<String>,
    pub exit_code: Option<i32>,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ReportReceipt {
    pub bytes: u64,
    pub identity: String,
}

/// The database facts a `CodeQL` analysis is bound to. A database whose recorded facts disagree
/// with the current source snapshot, build configuration, query packs, or executable is stale.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DatabaseBinding {
    pub source_snapshot_identity: String,
    pub language: String,
    pub build_mode: String,
    pub query_packs: Vec<QueryPackReceipt>,
    pub executable_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DatabaseReceipt {
    pub path: String,
    pub created: bool,
    pub binding: DatabaseBinding,
    pub binding_identity: String,
    pub creation: Option<StepReceipt>,
    pub analysis: Option<StepReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Execution {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub producer: String,
    pub driver_name: String,
    pub intake_profile_id: String,
    pub intake_profile_identity: String,
    pub evidence_origin: String,
    pub evidence_tier: String,
    pub supported_languages: Vec<String>,
    pub executable: Option<ExecutableReceipt>,
    pub rules: RuleReceipt,
    pub query_packs: Vec<QueryPackReceipt>,
    pub scope: ScopeReceipt,
    pub rust: RustScopeReceipt,
    pub source_snapshot_identity: String,
    pub source_files: Vec<SourceFile>,
    pub limits: LimitReceipt,
    pub steps: Vec<StepReceipt>,
    pub database: Option<DatabaseReceipt>,
    pub report: Option<ReportReceipt>,
    pub outcome: String,
    pub diagnostics: Vec<String>,
}

pub(crate) mod admission;
mod analysis;
mod codeql;
pub(crate) mod command;
pub(crate) mod process;
pub(crate) mod recording;
pub(crate) mod selection;

pub(crate) use process::Executor;

#[cfg(test)]
mod tests {
    include!("producer_tests.rs");
}
