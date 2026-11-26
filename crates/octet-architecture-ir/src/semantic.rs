//! Semantic MIR body and effect observation records for bounded sequential Rust subsets.

use serde::Deserialize;
use serde::Serialize;

/// Schema version for per-unit compiler semantic MIR summaries.
pub const SEMANTIC_MIR_UNIT_SCHEMA_VERSION: &str = "octet-semantic-mir-unit/v1";

const SEMANTIC_UNIT_DOMAIN: &str = "octet.semantic-mir-unit/v1";

/// Compiler stage at which semantic MIR facts were extracted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MirStage {
    OptimizedMir,
}

/// Control-flow context for an implicit destructor edge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DropPathKind {
    Normal,
    UnwindCleanup,
    NormalAndUnwind,
}

/// Resolution classification for a MIR call terminator.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallResolutionKind {
    ConcreteLocal,
    ConcreteExternal,
    ConcreteTraitImpl,
    ParametricTraitCall,
    DynamicDispatch,
    UnresolvedFnPtr,
    ForeignFfi,
    Intrinsic,
}

/// Ownership and region boundary of a resolved memory write target.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryRegion {
    OwnedLocal,
    LocalField,
    ParamMutRef,
    EscapingAlias,
    InteriorMutable,
    RawPointer,
    StaticGlobal,
    UnknownRegion,
}

/// Unsupported or unmodeled construct observed during MIR collection.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnsupportedKind {
    AsyncCoroutine,
    UnsafeBlock,
    RawPointerDeref,
    InlineAsm,
    ForeignFfi,
    UnresolvedDrop,
    UnresolvedFnPtr,
    DynamicDispatch,
    UnwindCleanup,
    BudgetExhausted,
    Other,
}

/// Implicit destructor edge observed at a MIR `Drop` terminator.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DropEdge {
    pub dropped_type: String,
    pub destructor_definition: Option<String>,
    pub path_kind: DropPathKind,
    pub is_resolved: bool,
    pub has_effectful_candidate: bool,
    pub span_id: Option<String>,
}

/// Resolved or parametric call edge observed in MIR.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MirCallEdge {
    pub callee_definition: Option<String>,
    pub resolved_instance: Option<String>,
    pub trait_definition: Option<String>,
    pub resolution: CallResolutionKind,
    pub has_generic_params: bool,
    pub span_id: Option<String>,
}

/// Memory write classified by resolved compiler type and ownership region.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryWrite {
    pub local_index: u32,
    pub projection: String,
    pub resolved_type: String,
    pub region: MemoryRegion,
    pub is_observable_external: bool,
    pub is_interior_mutability: bool,
    pub span_id: Option<String>,
}

/// Unsupported construct or analysis-boundary marker inside a MIR body.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedConstruct {
    pub kind: UnsupportedKind,
    pub detail: String,
    pub span_id: Option<String>,
}

/// Semantic MIR observation summary for one local function or closure body.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BodyObservation {
    pub item_id: String,
    pub qualified_path: String,
    pub mir_stage: MirStage,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub has_unwind_edges: bool,
    pub implicit_panic_assert_count: u32,
    pub basic_block_count: usize,
    pub step_count: usize,
    pub drops: Vec<DropEdge>,
    pub calls: Vec<MirCallEdge>,
    pub memory_writes: Vec<MemoryWrite>,
    pub unsupported: Vec<UnsupportedConstruct>,
    pub partial_moves_observed: usize,
    pub ended_borrow_reassignments: usize,
    pub span_id: String,
}

/// Per-unit semantic MIR summary emitted by the compiler collector.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitSummary {
    pub schema_version: String,
    pub unit_id: String,
    pub package_name: String,
    pub target_name: String,
    pub compiler_stage: String,
    pub toolchain_blake3: String,
    pub source_blake3: String,
    pub bodies: Vec<BodyObservation>,
    pub is_budget_exhausted: bool,
    pub skipped_bodies: Vec<String>,
    pub summary_blake3: String,
}

/// Computes the canonical BLAKE3 identity over a semantic MIR unit summary.
///
/// # Errors
///
/// Returns an error if canonical serialization of `summary` fails.
pub fn compute_unit_summary_blake3(summary: &UnitSummary) -> Result<String, String> {
    let mut normalized = summary.clone();
    normalized.summary_blake3 = String::new();
    crate::identity::canonical_identity(SEMANTIC_UNIT_DOMAIN, &normalized)
}
