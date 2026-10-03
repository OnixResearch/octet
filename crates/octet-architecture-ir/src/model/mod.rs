pub const COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION: &str = "octet-compiler-architecture-shard/v2";
pub const COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION: &str = "octet-compiler-architecture-ir/v2";
pub const COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION: &str = "octet-compiler-architecture-coverage/v1";
pub const COMPILER_ARCHITECTURE_ISSUE_SCHEMA_VERSION: &str = "octet-compiler-architecture-issue/v1";
pub const COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION: &str = "octet-compiler-architecture-receipt/v1";
pub const COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_SCHEMA_VERSION: &str =
    "octet-compiler-architecture-collector-context/v3";
pub const COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV: &str = "OCTET_COMPILER_ARCHITECTURE_CONTEXT";
pub const COMPILER_ARCHITECTURE_CONTRACT_PATH: &str =
    "standards/compiler-architecture-ir/contracts/compiler-architecture-ir.generated.ncl";

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCollectionLimits {
    pub max_units: usize,
    pub max_shards: usize,
    pub max_modules: usize,
    pub max_items: usize,
    pub max_types: usize,
    pub max_traits: usize,
    pub max_implementations: usize,
    pub max_signatures: usize,
    pub max_calls: usize,
    pub max_constructors: usize,
    pub max_effects: usize,
    pub max_expansions: usize,
    pub max_configurations: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_type_shapes: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_function_safety_facts: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_provenance_facts: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_flow_bodies: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_flow_steps: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_flow_rounds: usize,
    pub max_unknowns: usize,
    pub max_spans: usize,
    pub max_span_bytes: usize,
    pub max_shard_bytes: usize,
    pub max_artifact_bytes: usize,
}

const fn default_authority_fact_limit() -> usize {
    1
}

fn empty_string_vec() -> Vec<String> {
    Vec::new()
}

fn empty_value_flow_selections() -> Vec<CompilerValueFlowSelection> {
    Vec::new()
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
pub enum CompilerArchitectureRole {
    DomainCore,
    ApplicationShell,
    InboundAdapter,
    OutboundAdapter,
    CompositionRoot,
    Test,
    Generated,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilationUnit {
    pub unit_id: String,
    pub package_name: String,
    pub package_version: String,
    pub target_name: String,
    pub target_kind: String,
    /// Exact workspace-relative Cargo target root observed by the compiler adapter.
    pub source_path: String,
    pub crate_types: Vec<String>,
    pub target_triple: String,
    pub features: Vec<String>,
    pub default_features: bool,
    pub role: CompilerArchitectureRole,
    pub toolchain_blake3: String,
    pub collector_blake3: String,
    pub policy_blake3: String,
    pub source_blake3: String,
    pub invocation_blake3: String,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SourceOrigin {
    Workspace { source_path: String },
    ExternalGenerated { provenance_blake3: String },
}

#[derive(Clone, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub span_id: String,
    pub origin: SourceOrigin,
    pub byte_start: u64,
    pub byte_end: u64,
    pub line_start: u32,
    pub column_start: u32,
    pub from_expansion: bool,
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
pub enum ItemKind {
    Module,
    Function,
    Struct,
    Enum,
    Union,
    Trait,
    TypeAlias,
    Implementation,
    Constant,
    Static,
    Foreign,
    Other,
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
pub enum TypeKind {
    Primitive,
    Adt,
    Tuple,
    Reference,
    Pointer,
    Function,
    TraitObject,
    Opaque,
    Generic,
    Other,
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
pub enum DispatchKind {
    Static,
    Trait,
    Dynamic,
    Foreign,
    Intrinsic,
    Unknown,
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
pub enum CapabilityFamily {
    Time,
    Randomness,
    Filesystem,
    Network,
    Process,
    Environment,
    Secrets,
    UnsafeCode,
    GlobalState,
    Concurrency,
    Observability,
    Ffi,
    Allocation,
    Unknown,
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
pub enum ValueProvenanceStatus {
    Exact,
    Merged,
    Unknown,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerValueFlowSelection {
    pub tracked_callee_definitions: Vec<String>,
    pub tracked_type_definitions: Vec<String>,
    #[serde(default = "empty_string_vec")]
    pub root_origin_definitions: Vec<String>,
    #[serde(default = "empty_string_vec")]
    pub propagation_definitions: Vec<String>,
}

mod facts;
mod fault;
mod records;

pub use facts::*;
pub use fault::UnitSelectionError;
pub use records::*;
