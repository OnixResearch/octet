use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::ObjectArgs;
use crate::cli::ObjectMode;
use crate::cli::OutputFormat;
use crate::cli::SupplementFamilyArg;
use crate::cli::SupplementScopeArg;
use crate::determinism;
use crate::fingerprint;

const FUNCTION_OBJECT_SCHEMA: &str = "octet.function-object.v1";
#[allow(dead_code)]
const HIR_SUPPLEMENT_SCHEMA: &str = "octet.function-object-hir-supplement.v1";
#[allow(dead_code)]
const MIR_SUPPLEMENT_SCHEMA: &str = "octet.function-object-mir-supplement.v1";
const THIR_SUPPLEMENT_SCHEMA: &str = "octet.function-object-thir-supplement.v1";
const RUSTC_HIR_SPIKE_EXTRACTOR_NAME: &str = "octet-rustc-hir-fixture-spike";
const RUSTC_HIR_SPIKE_EXTRACTOR_VERSION: &str = env!("CARGO_PKG_VERSION");
const RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH: &str = "cargo-octet/tests/fixtures/hir-spike/checked.rs";
const RUSTC_HIR_SPIKE_FUNCTION: &str = "checked";
const RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH: &str = "cargo-octet/tests/fixtures/hir-spike/clamp.rs";
const RUSTC_HIR_SPIKE_BRANCH_FUNCTION: &str = "clamp_to_limit";
#[allow(dead_code)]
const SUPPLEMENT_SCHEMA_VERSION: u32 = 1;
const CORPUS_RECEIPT_SCHEMA: &str = "octet.function-object-corpus-receipt.v1";
const CORPUS_CHECK_REPORT_SCHEMA: &str = "octet.function-object-corpus-check-report.v1";
const CORPUS_RECEIPT_SCHEMA_VERSION: u32 = 1;
const NORMALIZATION_ALGORITHM: &str = "octet-source-token-v1";
#[allow(dead_code)]
const SUPPLEMENT_STATUS_VALUES: &[&str] = &["unsupported-planned", "pilot-supported", "unknown", "unsupported"];
const CORPUS_RECEIPT_REQUIRED_FIELDS: &[&str] = &[
    "schema",
    "schema_version",
    "mode",
    "object_count",
    "source_paths",
    "object_set_hash",
    "caveat_counts",
    "pure_cache_blocked_count",
    "replay",
    "boundary",
];
const CORPUS_CHECK_REPORT_REQUIRED_FIELDS: &[&str] = &[
    "schema",
    "schema_version",
    "mode",
    "status",
    "receipt_path",
    "comparisons",
    "errors",
];
const CORPUS_CHECK_COMPARISON_FIELDS: &[&str] = &["field", "status", "expected", "current"];
const CORPUS_CHECK_ERROR_FIELDS: &[&str] = &["kind", "field", "message"];
const CORPUS_RECEIPT_COMPARED_FIELDS: &[&str] = &[
    "schema",
    "schema_version",
    "object_count",
    "source_paths",
    "object_set_hash",
    "caveat_counts",
    "pure_cache_blocked_count",
    "boundary",
];
const DEPENDENCY_STATUS_MACRO_CONTEXT: &str = "unsupported-macro-context";
const OBJECT_MARKER: &str = "// octet-object:";
const HASH_PREFIX: &str = "b3:";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FunctionObjectV1 {
    pub schema: &'static str,
    pub object_hash: String,
    pub semantic_id: String,
    pub name: String,
    pub source_path: String,
    pub source_span: SourceSpan,
    pub body_hash: String,
    pub normalized_body_hash: String,
    pub signature_hash: String,
    pub dependency_set_hash: String,
    pub dependency_caveats: Vec<DependencyCaveat>,
    pub effect_caveats: Vec<EffectCaveat>,
    pub macro_caveats: Vec<MacroCaveat>,
    pub normalization: NormalizationMetadata,
    pub caveat_summary: CaveatSummary,
    pub pure_cache_eligibility: PureCacheEligibility,
    pub correctness_claim: CorrectnessClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ObjectSupplementStatus {
    UnsupportedPlanned,
    PilotSupported,
    Unknown,
    Unsupported,
}

impl ObjectSupplementStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedPlanned => "unsupported-planned",
            Self::PilotSupported => "pilot-supported",
            Self::Unknown => "unknown",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ObjectSupplementFamily {
    Hir,
    Mir,
}

impl ObjectSupplementFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::Hir => "hir",
            Self::Mir => "mir",
        }
    }

    fn schema(self) -> &'static str {
        match self {
            Self::Hir => HIR_SUPPLEMENT_SCHEMA,
            Self::Mir => MIR_SUPPLEMENT_SCHEMA,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PilotSupplementContext {
    rustc_version: String,
    target_triple: String,
    extractor_name: &'static str,
    extractor_version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct FunctionObjectSupplementV1 {
    schema: &'static str,
    schema_version: u32,
    family: &'static str,
    status: &'static str,
    function_object_hash: String,
    source_path: String,
    function_name: String,
    rustc_version: String,
    target_triple: String,
    extractor: SupplementExtractorIdentity,
    observation_hash: String,
    observations: BTreeMap<String, SupplementObservation>,
    extraction_caveats: Vec<SupplementCaveat>,
    boundary: SupplementBoundary,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct SupplementExtractorIdentity {
    name: &'static str,
    version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct SupplementObservation {
    status: &'static str,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct SupplementCaveat {
    family: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct SupplementBoundary {
    relationship_to_v1: &'static str,
    identity_scope: &'static str,
    claims_correctness: bool,
    claims_semantic_equivalence: bool,
    claims_cross_toolchain_stability: bool,
    claims_pure_cache_eligibility: bool,
    detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SourceSpan {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DependencyCaveat {
    pub kind: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EffectCaveat {
    pub kind: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MacroCaveat {
    pub kind: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NormalizationMetadata {
    pub algorithm: &'static str,
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub hash_algorithm: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CorrectnessClaim {
    pub claims_correctness: bool,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CaveatSummary {
    pub dependency_identity: &'static str,
    pub macro_expansion: &'static str,
    pub effect_summary: &'static str,
    pub cache_policy: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PureCacheEligibility {
    pub eligible: bool,
    pub status: &'static str,
    pub blockers: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct FunctionObjectReport {
    schema_version: u32,
    mode: &'static str,
    objects: Vec<FunctionObjectV1>,
}

#[derive(Debug, Clone)]
struct ObjectRecord {
    object: FunctionObjectV1,
}

#[derive(Debug, Clone)]
struct ObjectMarker {
    start: usize,
    end: usize,
    stored_hash: String,
    review: Option<String>,
    evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NamePointerManifest {
    schema_version: u32,
    pointers: BTreeMap<String, NamePointer>,
    #[serde(default)]
    history: BTreeMap<String, Vec<NamePointerHistory>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NamePointer {
    object_hash: String,
    semantic_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NamePointerHistory {
    old_hash: String,
    new_hash: String,
    semantic_name: String,
    diff_summary: String,
    rollback_handle: String,
}

#[derive(Debug, Clone, Serialize)]
struct PointerReceipt {
    schema_version: u32,
    semantic_name: String,
    old_hash: Option<String>,
    new_hash: String,
    diff_summary: String,
    rollback_handle: String,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectMarkerReport {
    schema_version: u32,
    mode: &'static str,
    records: Vec<ObjectMarkerRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusReceipt {
    schema: &'static str,
    schema_version: u32,
    mode: &'static str,
    object_count: usize,
    source_paths: Vec<String>,
    object_set_hash: String,
    caveat_counts: ObjectCorpusCaveatCounts,
    pure_cache_blocked_count: usize,
    replay: CorpusReplay,
    boundary: CorpusBoundary,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusCaveatCounts {
    dependency_status: BTreeMap<String, usize>,
    effect_kind: BTreeMap<String, usize>,
    macro_status: BTreeMap<String, usize>,
    pure_cache_blocker: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusCaveatTaxonomyReport {
    schema: &'static str,
    schema_version: u32,
    mode: &'static str,
    object_count: usize,
    source_paths: Vec<String>,
    source_buckets: Vec<ObjectCorpusCaveatSourceBucket>,
    caveat_counts: ObjectCorpusCaveatCounts,
    pure_cache_blocked_count: usize,
    boundary: CorpusBoundary,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusCaveatSourceBucket {
    source_path: String,
    object_count: usize,
    caveat_counts: ObjectCorpusCaveatCounts,
    pure_cache_blocked_count: usize,
    blocker_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CorpusReplay {
    command: String,
    tool_name: &'static str,
    tool_version: &'static str,
    determinism_note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct CorpusBoundary {
    identity_scope: &'static str,
    claims_correctness: bool,
    claims_semantic_equivalence: bool,
    detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusCheckReport {
    schema: &'static str,
    schema_version: u32,
    mode: &'static str,
    status: &'static str,
    receipt_path: String,
    comparisons: Vec<ObjectCorpusCheckComparison>,
    errors: Vec<ObjectCorpusCheckError>,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusCheckComparison {
    field: &'static str,
    status: &'static str,
    expected: serde_json::Value,
    current: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusCheckError {
    kind: String,
    field: Option<String>,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusContractReport {
    schema: &'static str,
    schema_version: u32,
    mode: &'static str,
    source_of_truth: &'static str,
    receipt: ObjectCorpusSurfaceContract,
    check_report: ObjectCorpusSurfaceContract,
    supplemental_evidence: Vec<ObjectSupplementalEvidenceContract>,
    boundaries: ObjectCorpusContractBoundaries,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusSurfaceContract {
    schema: &'static str,
    schema_version: u32,
    required_fields: Vec<&'static str>,
    compared_fields: Vec<&'static str>,
    status_values: Vec<&'static str>,
    comparison_fields: Vec<&'static str>,
    error_fields: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectSupplementalEvidenceContract {
    family: &'static str,
    schema: &'static str,
    schema_version: u32,
    status: &'static str,
    relationship_to_v1: &'static str,
    toolchain_scope: &'static str,
    required_fields: Vec<&'static str>,
    unsupported_reason: &'static str,
    required_evidence_before_support: Vec<&'static str>,
    extractor_identity_expectations: Vec<&'static str>,
    selected_fixture_shapes: Vec<&'static str>,
    fixture_replay_command: &'static str,
    claims_correctness: bool,
    claims_semantic_equivalence: bool,
    claims_cross_toolchain_stability: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectCorpusContractBoundaries {
    identity_scope: &'static str,
    claims_correctness: bool,
    claims_semantic_equivalence: bool,
    determinism_scope: &'static str,
    detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectBundle {
    schema: &'static str,
    schema_version: u32,
    bundle_hash: String,
    manifest_hash: String,
    object_count: usize,
    pointer_count: usize,
    replay: BundleReplay,
    objects: Vec<serde_json::Value>,
    pointers: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct BundleReplay {
    command: String,
    tool_name: &'static str,
    tool_version: &'static str,
    determinism_note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ObjectMarkerRecord {
    path: String,
    name: String,
    stored_hash: Option<String>,
    computed_hash: String,
    status: &'static str,
    review: Option<String>,
    evidence: Option<String>,
}

pub fn run(args: ObjectArgs) -> Result<i32, String> {
    match args.mode.clone() {
        ObjectMode::Emit => run_emit(args),
        ObjectMode::Check => run_marker(args, false),
        ObjectMode::Update => run_marker(args, true),
        ObjectMode::Diff {
            old,
            new,
            pointer_manifest,
        } => run_diff(&old, &new, pointer_manifest.as_deref(), args.output_format),
        ObjectMode::PointerSet {
            manifest,
            name,
            hash,
            objects,
            receipt,
        } => run_pointer_set(&manifest, &name, &hash, &objects, &receipt),
        ObjectMode::PointerValidate { manifest, objects } => run_pointer_validate(&manifest, &objects),
        ObjectMode::BundleExport {
            objects,
            pointers,
            output,
            replay_command,
        } => run_bundle_export(&objects, pointers.as_deref(), &output, &replay_command),
        ObjectMode::CorpusReceipt { output } => run_corpus_receipt(&args.paths, &output),
        ObjectMode::CorpusCheck { receipt } => run_corpus_check(&args.paths, &receipt, args.output_format),
        ObjectMode::CorpusCaveats => run_corpus_caveats(&args.paths, args.output_format),
        ObjectMode::CorpusContract => run_corpus_contract(args.output_format),
        ObjectMode::SupplementEmit { family, scope } => {
            run_supplement_emit(&args.paths, args.function.as_deref(), family, scope, args.output_format)
        }
        ObjectMode::SupplementCheck { family, scope, receipt } => {
            run_supplement_check(&args.paths, args.function.as_deref(), family, scope, &receipt, args.output_format)
        }
    }
}

fn run_emit(args: ObjectArgs) -> Result<i32, String> {
    let objects = collect_objects(&args.paths, args.function.as_deref())?
        .into_iter()
        .map(|record| record.object)
        .collect::<Vec<_>>();

    if objects.is_empty() {
        return Err("no functions matched object emission selection".to_owned());
    }

    let report = FunctionObjectReport {
        schema_version: 1,
        mode: "emit",
        objects,
    };
    match args.output_format {
        OutputFormat::Human => print_human(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => return Err("object emit does not support SARIF output; use human or json".to_owned()),
    }
    Ok(0)
}

fn run_corpus_receipt(paths: &[PathBuf], output: &Path) -> Result<i32, String> {
    let objects = collect_objects(paths, None)?.into_iter().map(|record| record.object).collect::<Vec<_>>();
    if objects.is_empty() {
        return Err("no functions matched object corpus receipt selection".to_owned());
    }

    let receipt = corpus_receipt(paths, &objects);
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&receipt).map_err(|error| format!("serialize corpus receipt: {error}"))?;
    fs::write(output, format!("{json}\n")).map_err(|error| format!("write {}: {error}", output.display()))?;
    println!("wrote object corpus receipt: {}", output.display());
    Ok(0)
}

fn run_corpus_caveats(paths: &[PathBuf], output_format: OutputFormat) -> Result<i32, String> {
    let objects = collect_objects(paths, None)?.into_iter().map(|record| record.object).collect::<Vec<_>>();
    if objects.is_empty() {
        return Err("no functions matched object corpus caveat taxonomy selection".to_owned());
    }
    let report = corpus_caveat_taxonomy_report(&objects);
    match output_format {
        OutputFormat::Human => print_corpus_caveat_taxonomy_human(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => {
            return Err("object corpus caveats does not support SARIF output; use human or json".to_owned());
        }
    }
    Ok(0)
}

fn run_corpus_contract(output_format: OutputFormat) -> Result<i32, String> {
    if matches!(output_format, OutputFormat::Sarif) {
        return Err("object corpus contract does not support SARIF output; use human or json".to_owned());
    }
    let report = object_corpus_contract_report();
    match output_format {
        OutputFormat::Human => print_corpus_contract_human(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => unreachable!("SARIF rejected above"),
    }
    Ok(0)
}

fn run_supplement_emit(
    paths: &[PathBuf],
    function: Option<&str>,
    family: SupplementFamilyArg,
    scope: SupplementScopeArg,
    output_format: OutputFormat,
) -> Result<i32, String> {
    if matches!(output_format, OutputFormat::Sarif) {
        return Err("object supplement emit does not support SARIF output; use human or json".to_owned());
    }
    let supplements = supplement_records(paths, function, family, scope)?;
    if supplements.is_empty() {
        return Err("no functions matched supplement emission selection".to_owned());
    }
    let report = serde_json::json!({
        "schema_version": SUPPLEMENT_SCHEMA_VERSION,
        "mode": "supplement-emit",
        "family": family.as_str(),
        "scope": scope.as_str(),
        "supplements": supplements,
        "boundary": supplement_boundary(),
    });
    print_supplement_emit_report(&report, output_format)?;
    Ok(0)
}

fn run_supplement_check(
    paths: &[PathBuf],
    function: Option<&str>,
    family: SupplementFamilyArg,
    scope: SupplementScopeArg,
    receipt_path: &Path,
    output_format: OutputFormat,
) -> Result<i32, String> {
    if matches!(output_format, OutputFormat::Sarif) {
        return Err("object supplement check does not support SARIF output; use human or json".to_owned());
    }
    let expected = read_supplement_receipt(receipt_path)?;
    let current = supplement_records(paths, function, family, scope)?;
    let current_first = current.first().ok_or_else(|| "no functions matched supplement check selection".to_owned())?;
    let comparisons = supplement_comparisons(&expected, current_first);
    let status = if comparisons.iter().all(|comparison| comparison.status == "match") {
        "pass"
    } else {
        "fail"
    };
    let report = SupplementCheckReport {
        schema_version: SUPPLEMENT_SCHEMA_VERSION,
        mode: "supplement-check",
        family: family.as_str(),
        scope: scope.as_str(),
        status,
        receipt_path: determinism::normalize_path(receipt_path),
        comparisons,
        boundary: supplement_boundary(),
    };
    print_supplement_check_report(&report, output_format)?;
    Ok(if status == "pass" { 0 } else { 1 })
}

fn supplement_records(
    paths: &[PathBuf],
    function: Option<&str>,
    family: SupplementFamilyArg,
    scope: SupplementScopeArg,
) -> Result<Vec<FunctionObjectSupplementV1>, String> {
    match scope {
        SupplementScopeArg::Pilot => {
            let context = runtime_pilot_context(family);
            collect_objects(paths, function)?
                .into_iter()
                .map(|record| match family {
                    SupplementFamilyArg::Hir => Ok(pilot_hir_supplement(&record.object, &context)),
                    SupplementFamilyArg::Mir => Ok(pilot_mir_supplement(&record.object, &context)),
                })
                .collect()
        }
        SupplementScopeArg::RustcHirFixtureSpike => {
            if family != SupplementFamilyArg::Hir {
                return Err("rustc HIR fixture spike only supports `--family hir`".to_owned());
            }
            let function = function.unwrap_or(RUSTC_HIR_SPIKE_FUNCTION);
            let fixture_path = rustc_hir_spike_cli_fixture_path(paths)?;
            Ok(vec![rustc_hir_spike_supplement(&fixture_path, function)?])
        }
    }
}

fn rustc_hir_spike_cli_fixture_path(paths: &[PathBuf]) -> Result<PathBuf, String> {
    if paths.is_empty() || paths == [PathBuf::from(".")] {
        return Ok(PathBuf::from(RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH));
    }
    if paths.len() != 1 {
        return Err("rustc HIR fixture spike accepts exactly one fixture path".to_owned());
    }
    Ok(paths[0].clone())
}

fn runtime_pilot_context(family: SupplementFamilyArg) -> PilotSupplementContext {
    let rustc_version = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "unknown-rustc-toolchain".to_owned());
    let target_triple = std::env::var("TARGET")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));
    let extractor_name = match family {
        SupplementFamilyArg::Hir => "octet-hir-pilot-source-token",
        SupplementFamilyArg::Mir => "octet-mir-pilot-source-token",
    };
    PilotSupplementContext {
        rustc_version,
        target_triple,
        extractor_name,
        extractor_version: env!("CARGO_PKG_VERSION"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RustcHirSpikeFixture {
    relative_path: &'static str,
    function: &'static str,
    shape: &'static str,
}

const RUSTC_HIR_SPIKE_FIXTURES: &[RustcHirSpikeFixture] = &[
    RustcHirSpikeFixture {
        relative_path: RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH,
        function: RUSTC_HIR_SPIKE_FUNCTION,
        shape: "checked-arithmetic",
    },
    RustcHirSpikeFixture {
        relative_path: RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH,
        function: RUSTC_HIR_SPIKE_BRANCH_FUNCTION,
        shape: "branch-control-flow",
    },
];

fn rustc_hir_spike_fixture_for(fixture_path: &Path, function: &str) -> Result<RustcHirSpikeFixture, String> {
    let normalized_path = determinism::normalize_path(fixture_path);
    RUSTC_HIR_SPIKE_FIXTURES
        .iter()
        .copied()
        .find(|fixture| fixture.relative_path == normalized_path && fixture.function == function)
        .ok_or_else(|| {
            let allowed = RUSTC_HIR_SPIKE_FIXTURES
                .iter()
                .map(|fixture| format!("{}::{}", fixture.relative_path, fixture.function))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "rustc HIR spike is fixture-bounded to checked-in fixture/function pairs [{allowed}], got {normalized_path}::{function}"
            )
        })
}

#[allow(dead_code)]
fn rustc_hir_spike_supplement(fixture_path: &Path, function: &str) -> Result<FunctionObjectSupplementV1, String> {
    let fixture = rustc_hir_spike_fixture_for(fixture_path, function)?;

    let read_path = rustc_hir_spike_read_path(fixture_path);
    let source = fs::read_to_string(&read_path).map_err(|error| format!("read {}: {error}", read_path.display()))?;
    let span = fingerprint::find_functions(&source)
        .into_iter()
        .find(|span| span.name == function)
        .ok_or_else(|| format!("fixture {} does not contain function `{function}`", read_path.display()))?;
    let object = function_object_for_source(fixture_path, &source, &span, &source[span.start..span.end])?;
    let hir_tree = rustc_hir_tree(&read_path)?;
    let context = runtime_rustc_hir_spike_context()?;
    let hir_tree_hash = determinism::full_hash(&hir_tree);
    let function_owner_marker = format!("::{function}");

    let mut observations = BTreeMap::new();
    observations.insert("fixture_context".to_owned(), SupplementObservation {
        status: ObjectSupplementStatus::PilotSupported.as_str(),
        detail: format!(
            "fixture={}; function={function}; fixture_shape={}; object_hash={}",
            determinism::normalize_path(fixture_path),
            fixture.shape,
            object.object_hash
        ),
    });
    observations.insert("rustc_hir_tree_hash".to_owned(), SupplementObservation {
        status: ObjectSupplementStatus::PilotSupported.as_str(),
        detail: format!("rustc -Z unpretty=hir-tree output hash={hir_tree_hash}"),
    });
    observations.insert("function_owner_marker".to_owned(), SupplementObservation {
        status: if hir_tree.contains(&function_owner_marker) {
            ObjectSupplementStatus::PilotSupported.as_str()
        } else {
            ObjectSupplementStatus::Unsupported.as_str()
        },
        detail: format!("HIR tree contains selected owner marker `{function_owner_marker}`"),
    });
    observations.insert("support_boundary".to_owned(), SupplementObservation {
        status: ObjectSupplementStatus::Unsupported.as_str(),
        detail:
            "rustc HIR spike is limited to selected checked-in fixture shapes and does not claim general HIR support"
                .to_owned(),
    });

    let mut supplement = supplement_for_object(ObjectSupplementFamily::Hir, &object, &context, observations);
    supplement.extraction_caveats.push(SupplementCaveat {
        family: ObjectSupplementFamily::Hir.as_str(),
        status: ObjectSupplementStatus::Unsupported.as_str(),
        detail: "rustc-backed HIR observation is fixture-bounded and derived from rustc -Z unpretty=hir-tree; it is not a correctness or semantic-equivalence proof".to_owned(),
    });
    Ok(supplement)
}

#[allow(dead_code)]
fn runtime_rustc_hir_spike_context() -> Result<PilotSupplementContext, String> {
    let version_output = Command::new(rustc_command())
        .arg("-vV")
        .output()
        .map_err(|error| format!("run rustc -vV: {error}"))?;
    if !version_output.status.success() {
        return Err(format!("rustc -vV failed: {}", String::from_utf8_lossy(&version_output.stderr).trim()));
    }
    let version_text = String::from_utf8_lossy(&version_output.stdout).trim().to_owned();
    let target_triple = version_text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap_or("unknown-rustc-host")
        .to_owned();
    Ok(PilotSupplementContext {
        rustc_version: version_text,
        target_triple,
        extractor_name: RUSTC_HIR_SPIKE_EXTRACTOR_NAME,
        extractor_version: RUSTC_HIR_SPIKE_EXTRACTOR_VERSION,
    })
}

#[allow(dead_code)]
fn rustc_hir_spike_read_path(fixture_path: &Path) -> PathBuf {
    if fixture_path.exists() {
        return fixture_path.to_path_buf();
    }
    fixture_path
        .strip_prefix("cargo-octet")
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| fixture_path.to_path_buf())
}

#[allow(dead_code)]
fn rustc_hir_tree(fixture_path: &Path) -> Result<String, String> {
    let output = Command::new(rustc_command())
        .args(["-Z", "unpretty=hir-tree", "--crate-type", "lib"])
        .arg(fixture_path)
        .output()
        .map_err(|error| format!("run rustc HIR tree extractor for {}: {error}", fixture_path.display()))?;
    if !output.status.success() {
        return Err(format!(
            "rustc HIR tree extractor failed for {}: {}",
            fixture_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("rustc HIR output was not UTF-8: {error}"))
}

#[allow(dead_code)]
fn rustc_command() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned())
}

#[derive(Debug, Clone, Serialize)]
struct SupplementCheckReport {
    schema_version: u32,
    mode: &'static str,
    family: &'static str,
    scope: &'static str,
    status: &'static str,
    receipt_path: String,
    comparisons: Vec<SupplementComparison>,
    boundary: SupplementBoundary,
}

#[derive(Debug, Clone, Serialize)]
struct SupplementComparison {
    field: &'static str,
    status: &'static str,
    expected: serde_json::Value,
    current: serde_json::Value,
}

fn read_supplement_receipt(path: &Path) -> Result<serde_json::Value, String> {
    let value = read_json(path)?;
    if let Some(supplements) = value.get("supplements").and_then(serde_json::Value::as_array) {
        return supplements.first().cloned().ok_or_else(|| format!("{} has no supplements", path.display()));
    }
    Ok(value)
}

const SUPPLEMENT_REPLAY_COMPARED_FIELDS: &[&str] = &[
    "schema",
    "schema_version",
    "family",
    "status",
    "function_object_hash",
    "source_path",
    "function_name",
    "rustc_version",
    "target_triple",
    "extractor",
    "observations",
    "observation_hash",
    "extraction_caveats",
    "boundary",
];

fn supplement_comparisons(
    expected: &serde_json::Value,
    current: &FunctionObjectSupplementV1,
) -> Vec<SupplementComparison> {
    let current = serde_json::to_value(current).expect("serialize current supplement");
    SUPPLEMENT_REPLAY_COMPARED_FIELDS
        .iter()
        .copied()
        .map(|field| {
            let expected_value = expected.get(field).cloned().unwrap_or(serde_json::Value::Null);
            let current_value = current.get(field).cloned().unwrap_or(serde_json::Value::Null);
            SupplementComparison {
                field,
                status: if expected_value == current_value {
                    "match"
                } else {
                    "mismatch"
                },
                expected: expected_value,
                current: current_value,
            }
        })
        .collect()
}

fn print_supplement_emit_report(report: &serde_json::Value, output_format: OutputFormat) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!("octet object supplement emit family={}", report["family"].as_str().unwrap_or("unknown"));
            println!("scope={}", report["scope"].as_str().unwrap_or("unknown"));
            let count = report["supplements"].as_array().map_or(0, Vec::len);
            println!("summary total={count}");
            println!("claims_correctness=false");
            println!("claims_semantic_equivalence=false");
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => unreachable!("SARIF rejected before supplement emit output"),
    }
    Ok(())
}

fn print_supplement_check_report(report: &SupplementCheckReport, output_format: OutputFormat) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet object supplement check status={} family={} scope={}",
                report.status, report.family, report.scope
            );
            println!("receipt_path={}", report.receipt_path);
            for comparison in &report.comparisons {
                println!("{}={}", comparison.field, comparison.status);
            }
            println!("claims_correctness={}", report.boundary.claims_correctness);
            println!("claims_semantic_equivalence={}", report.boundary.claims_semantic_equivalence);
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => unreachable!("SARIF rejected before supplement check output"),
    }
    Ok(())
}

#[allow(dead_code)]
fn pilot_hir_supplement(object: &FunctionObjectV1, context: &PilotSupplementContext) -> FunctionObjectSupplementV1 {
    let mut observations = BTreeMap::new();
    observations.insert("hir_context".to_owned(), SupplementObservation {
        status: ObjectSupplementStatus::PilotSupported.as_str(),
        detail: format!("source-token object {} is attached to a bounded HIR pilot context", object.object_hash),
    });
    observations.insert("macro_origin".to_owned(), SupplementObservation {
        status: caveat_status(object.macro_caveats.is_empty()),
        detail: caveat_detail(
            object.macro_caveats.is_empty(),
            "no macro-origin caveats observed by source scanner",
            "macro-origin detail remains source-scanner bounded",
        ),
    });
    observations.insert("dependency_identity".to_owned(), SupplementObservation {
        status: ObjectSupplementStatus::Unsupported.as_str(),
        detail: "resolved HIR dependency identity is outside this bounded pilot".to_owned(),
    });
    supplement_for_object(ObjectSupplementFamily::Hir, object, context, observations)
}

#[allow(dead_code)]
fn pilot_mir_supplement(object: &FunctionObjectV1, context: &PilotSupplementContext) -> FunctionObjectSupplementV1 {
    let mut observations = BTreeMap::new();
    for family in ["call", "panic", "drop", "async", "unsafe", "allocation", "effect"] {
        observations.insert(family.to_owned(), SupplementObservation {
            status: ObjectSupplementStatus::Unknown.as_str(),
            detail: format!("{family} summary is explicit unknown/unsupported in the bounded MIR pilot"),
        });
    }
    observations.insert("mir_context".to_owned(), SupplementObservation {
        status: ObjectSupplementStatus::PilotSupported.as_str(),
        detail: "MIR observation context is version-scoped and does not claim semantic equivalence".to_owned(),
    });
    supplement_for_object(ObjectSupplementFamily::Mir, object, context, observations)
}

#[allow(dead_code)]
fn supplement_for_object(
    family: ObjectSupplementFamily,
    object: &FunctionObjectV1,
    context: &PilotSupplementContext,
    observations: BTreeMap<String, SupplementObservation>,
) -> FunctionObjectSupplementV1 {
    let observation_hash = supplement_observation_hash(family, object, context, &observations);
    FunctionObjectSupplementV1 {
        schema: family.schema(),
        schema_version: SUPPLEMENT_SCHEMA_VERSION,
        family: family.as_str(),
        status: ObjectSupplementStatus::PilotSupported.as_str(),
        function_object_hash: object.object_hash.clone(),
        source_path: object.source_path.clone(),
        function_name: object.name.clone(),
        rustc_version: context.rustc_version.clone(),
        target_triple: context.target_triple.clone(),
        extractor: SupplementExtractorIdentity {
            name: context.extractor_name,
            version: context.extractor_version,
        },
        observation_hash,
        observations,
        extraction_caveats: supplement_caveats(family, object),
        boundary: supplement_boundary(),
    }
}

#[allow(dead_code)]
fn supplement_observation_hash(
    family: ObjectSupplementFamily,
    object: &FunctionObjectV1,
    context: &PilotSupplementContext,
    observations: &BTreeMap<String, SupplementObservation>,
) -> String {
    let mut lines = vec![
        family.schema().to_owned(),
        object.object_hash.clone(),
        object.source_path.clone(),
        object.name.clone(),
        context.rustc_version.clone(),
        context.target_triple.clone(),
        context.extractor_name.to_owned(),
        context.extractor_version.to_owned(),
    ];
    for (name, observation) in observations {
        lines.push(format!("{name}:{}:{}", observation.status, observation.detail));
    }
    determinism::full_hash(&lines.join("\n"))
}

#[allow(dead_code)]
fn supplement_caveats(family: ObjectSupplementFamily, object: &FunctionObjectV1) -> Vec<SupplementCaveat> {
    let mut caveats = Vec::new();
    if !object.dependency_caveats.is_empty() {
        caveats.push(SupplementCaveat {
            family: family.as_str(),
            status: ObjectSupplementStatus::Unsupported.as_str(),
            detail: "resolved dependency identity remains unsupported in supplement pilot".to_owned(),
        });
    }
    if !object.macro_caveats.is_empty() {
        caveats.push(SupplementCaveat {
            family: family.as_str(),
            status: ObjectSupplementStatus::Unsupported.as_str(),
            detail: "macro-expanded identity remains unsupported in supplement pilot".to_owned(),
        });
    }
    if caveats.is_empty() {
        caveats.push(SupplementCaveat {
            family: family.as_str(),
            status: ObjectSupplementStatus::Unknown.as_str(),
            detail: "pilot supplement remains source-scanner bounded and is not a proof".to_owned(),
        });
    }
    caveats
}

#[allow(dead_code)]
fn supplement_boundary() -> SupplementBoundary {
    SupplementBoundary {
        relationship_to_v1: "attached-supplement-preserves-function-object-v1-identity",
        identity_scope: "review-and-provenance-evidence",
        claims_correctness: false,
        claims_semantic_equivalence: false,
        claims_cross_toolchain_stability: false,
        claims_pure_cache_eligibility: false,
        detail: "HIR/MIR pilot supplements add bounded toolchain-scoped observations; Function Object v1 remains the review/provenance identity and no correctness or semantic-equivalence proof is implied.",
    }
}

#[allow(dead_code)]
fn canonical_supplement_json(supplement: &FunctionObjectSupplementV1) -> Result<String, String> {
    let mut json =
        serde_json::to_string_pretty(supplement).map_err(|error| format!("serialize supplement: {error}"))?;
    json.push('\n');
    Ok(json)
}

#[allow(dead_code)]
fn caveat_status(is_supported: bool) -> &'static str {
    if is_supported {
        ObjectSupplementStatus::PilotSupported.as_str()
    } else {
        ObjectSupplementStatus::Unknown.as_str()
    }
}

#[allow(dead_code)]
fn caveat_detail(is_supported: bool, supported: &'static str, unsupported: &'static str) -> String {
    if is_supported { supported } else { unsupported }.to_owned()
}

fn run_corpus_check(paths: &[PathBuf], receipt_path: &Path, output_format: OutputFormat) -> Result<i32, String> {
    if matches!(output_format, OutputFormat::Sarif) {
        return Err("object corpus check does not support SARIF output; use human or json".to_owned());
    }

    let receipt_path_string = determinism::normalize_path(receipt_path);
    let expected_text = match fs::read_to_string(receipt_path) {
        Ok(text) => text,
        Err(error) => {
            let report = invalid_corpus_check_report(receipt_path_string, vec![corpus_check_error(
                "read-receipt",
                Some("receipt"),
                format!("read {}: {error}", receipt_path.display()),
            )]);
            print_corpus_check_report(&report, output_format)?;
            return Ok(1);
        }
    };
    let expected: serde_json::Value = match serde_json::from_str(&expected_text) {
        Ok(value) => value,
        Err(error) => {
            let report = invalid_corpus_check_report(receipt_path_string, vec![corpus_check_error(
                "invalid-json",
                Some("receipt"),
                format!("parse {} as corpus receipt JSON: {error}", receipt_path.display()),
            )]);
            print_corpus_check_report(&report, output_format)?;
            return Ok(1);
        }
    };

    let mut errors = validate_corpus_receipt_shape(&expected);
    errors.extend(validate_corpus_source_selection(paths));
    if !errors.is_empty() {
        let report = invalid_corpus_check_report(receipt_path_string, errors);
        print_corpus_check_report(&report, output_format)?;
        return Ok(1);
    }

    let objects = match collect_objects(paths, None) {
        Ok(records) => records.into_iter().map(|record| record.object).collect::<Vec<_>>(),
        Err(error) => {
            let report = invalid_corpus_check_report(receipt_path_string, vec![corpus_check_error(
                "source-selection",
                Some("paths"),
                error,
            )]);
            print_corpus_check_report(&report, output_format)?;
            return Ok(1);
        }
    };
    if objects.is_empty() {
        let report = invalid_corpus_check_report(receipt_path_string, vec![corpus_check_error(
            "empty-corpus",
            Some("paths"),
            "no functions matched object corpus check selection",
        )]);
        print_corpus_check_report(&report, output_format)?;
        return Ok(1);
    }
    let current = serde_json::to_value(corpus_receipt(paths, &objects))
        .map_err(|error| format!("serialize current corpus receipt: {error}"))?;
    let comparisons = CORPUS_RECEIPT_COMPARED_FIELDS
        .iter()
        .copied()
        .map(|field| {
            let expected_value = expected.get(field).cloned().unwrap_or(serde_json::Value::Null);
            let current_value = current.get(field).cloned().unwrap_or(serde_json::Value::Null);
            ObjectCorpusCheckComparison {
                field,
                status: if expected_value == current_value {
                    "match"
                } else {
                    "mismatch"
                },
                expected: expected_value,
                current: current_value,
            }
        })
        .collect::<Vec<_>>();
    let status = if comparisons.iter().all(|comparison| comparison.status == "match") {
        "match"
    } else {
        "mismatch"
    };
    let report = ObjectCorpusCheckReport {
        schema: CORPUS_CHECK_REPORT_SCHEMA,
        schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
        mode: "corpus-check",
        status,
        receipt_path: receipt_path_string,
        comparisons,
        errors: Vec::new(),
    };
    print_corpus_check_report(&report, output_format)?;
    Ok(if status == "match" { 0 } else { 1 })
}

fn invalid_corpus_check_report(receipt_path: String, errors: Vec<ObjectCorpusCheckError>) -> ObjectCorpusCheckReport {
    ObjectCorpusCheckReport {
        schema: CORPUS_CHECK_REPORT_SCHEMA,
        schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
        mode: "corpus-check",
        status: "invalid",
        receipt_path,
        comparisons: Vec::new(),
        errors,
    }
}

fn corpus_check_error(
    kind: impl Into<String>,
    field: Option<&str>,
    message: impl Into<String>,
) -> ObjectCorpusCheckError {
    ObjectCorpusCheckError {
        kind: kind.into(),
        field: field.map(str::to_owned),
        message: message.into(),
    }
}

fn validate_corpus_receipt_shape(receipt: &serde_json::Value) -> Vec<ObjectCorpusCheckError> {
    let mut errors = Vec::new();
    let Some(object) = receipt.as_object() else {
        errors.push(corpus_check_error("invalid-receipt-shape", Some("receipt"), "receipt root must be a JSON object"));
        return errors;
    };

    for field in CORPUS_RECEIPT_REQUIRED_FIELDS {
        if !object.contains_key(*field) {
            errors.push(corpus_check_error(
                "missing-field",
                Some(field),
                format!("required field `{field}` is missing"),
            ));
        }
    }

    if let Some(schema) = receipt.get("schema").and_then(serde_json::Value::as_str)
        && schema != CORPUS_RECEIPT_SCHEMA
    {
        errors.push(corpus_check_error(
            "unsupported-schema",
            Some("schema"),
            format!("unsupported corpus receipt schema `{schema}`; expected `{CORPUS_RECEIPT_SCHEMA}`"),
        ));
    } else if receipt.get("schema").is_some_and(|value| !value.is_string()) {
        errors.push(corpus_check_error("invalid-field-type", Some("schema"), "schema must be a string"));
    }

    if let Some(version) = receipt.get("schema_version").and_then(serde_json::Value::as_u64)
        && version != u64::from(CORPUS_RECEIPT_SCHEMA_VERSION)
    {
        errors.push(corpus_check_error(
            "unsupported-schema-version",
            Some("schema_version"),
            format!(
                "unsupported corpus receipt schema_version `{version}`; expected `{CORPUS_RECEIPT_SCHEMA_VERSION}`"
            ),
        ));
    } else if receipt.get("schema_version").is_some_and(|value| !value.is_u64()) {
        errors.push(corpus_check_error(
            "invalid-field-type",
            Some("schema_version"),
            "schema_version must be an unsigned integer",
        ));
    }

    if let Some(paths) = receipt.get("source_paths").and_then(serde_json::Value::as_array) {
        if paths.is_empty() {
            errors.push(corpus_check_error(
                "empty-source-selection",
                Some("source_paths"),
                "source_paths must name at least one Rust source file",
            ));
        }
        let mut seen = BTreeSet::new();
        for path in paths {
            if let Some(path) = path.as_str() {
                if !seen.insert(path.to_owned()) {
                    errors.push(corpus_check_error(
                        "duplicate-source-path",
                        Some("source_paths"),
                        format!("receipt source path `{path}` appears more than once"),
                    ));
                }
            } else {
                errors.push(corpus_check_error(
                    "invalid-field-type",
                    Some("source_paths"),
                    "source_paths entries must be strings",
                ));
            }
        }
    } else if receipt.get("source_paths").is_some() {
        errors.push(corpus_check_error("invalid-field-type", Some("source_paths"), "source_paths must be an array"));
    }

    for field in ["object_count", "pure_cache_blocked_count"] {
        if receipt.get(field).is_some_and(|value| !value.is_u64()) {
            errors.push(corpus_check_error(
                "invalid-field-type",
                Some(field),
                format!("{field} must be an unsigned integer"),
            ));
        }
    }
    for field in ["object_set_hash", "mode"] {
        if receipt.get(field).is_some_and(|value| !value.is_string()) {
            errors.push(corpus_check_error("invalid-field-type", Some(field), format!("{field} must be a string")));
        }
    }
    for field in ["caveat_counts", "replay", "boundary"] {
        if receipt.get(field).is_some_and(|value| !value.is_object()) {
            errors.push(corpus_check_error("invalid-field-type", Some(field), format!("{field} must be an object")));
        }
    }

    errors
}

fn validate_corpus_source_selection(paths: &[PathBuf]) -> Vec<ObjectCorpusCheckError> {
    let mut errors = Vec::new();
    let mut seen = BTreeSet::new();
    for path in paths {
        let normalized = determinism::normalize_path(path);
        if !seen.insert(normalized.clone()) {
            errors.push(corpus_check_error(
                "duplicate-source-path",
                Some("paths"),
                format!("source path `{normalized}` appears more than once"),
            ));
        }
        if !path.exists() {
            errors.push(corpus_check_error(
                "source-path-missing",
                Some("paths"),
                format!("source path `{normalized}` does not exist"),
            ));
            continue;
        }
        if path.is_file() && path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            errors.push(corpus_check_error(
                "unsupported-source-path",
                Some("paths"),
                format!("source path `{normalized}` is not a Rust source file"),
            ));
        }
    }
    errors
}

fn object_corpus_contract_report() -> ObjectCorpusContractReport {
    ObjectCorpusContractReport {
        schema: "octet.object-corpus-contract-metadata.v1",
        schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
        mode: "corpus-contract",
        source_of_truth: "rust-dto-and-constants",
        receipt: ObjectCorpusSurfaceContract {
            schema: CORPUS_RECEIPT_SCHEMA,
            schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
            required_fields: CORPUS_RECEIPT_REQUIRED_FIELDS.to_vec(),
            compared_fields: CORPUS_RECEIPT_COMPARED_FIELDS.to_vec(),
            status_values: Vec::new(),
            comparison_fields: Vec::new(),
            error_fields: Vec::new(),
        },
        check_report: ObjectCorpusSurfaceContract {
            schema: CORPUS_CHECK_REPORT_SCHEMA,
            schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
            required_fields: CORPUS_CHECK_REPORT_REQUIRED_FIELDS.to_vec(),
            compared_fields: Vec::new(),
            status_values: vec!["match", "mismatch", "invalid"],
            comparison_fields: CORPUS_CHECK_COMPARISON_FIELDS.to_vec(),
            error_fields: CORPUS_CHECK_ERROR_FIELDS.to_vec(),
        },
        supplemental_evidence: supplemental_evidence_contracts(),
        boundaries: ObjectCorpusContractBoundaries {
            identity_scope: "review-and-provenance-evidence",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            determinism_scope: "bounded-local-determinism",
            detail: "Contract metadata describes JSON compatibility for object corpus evidence; it does not prove behavioral correctness, semantic equivalence, cross-machine determinism, dependency resolution, macro expansion identity, or pure-cache eligibility.",
        },
    }
}

fn supplemental_evidence_contracts() -> Vec<ObjectSupplementalEvidenceContract> {
    vec![
        ObjectSupplementalEvidenceContract {
            family: "hir",
            schema: HIR_SUPPLEMENT_SCHEMA,
            schema_version: SUPPLEMENT_SCHEMA_VERSION,
            status: ObjectSupplementStatus::PilotSupported.as_str(),
            relationship_to_v1: "optional-supplement-preserves-function-object-v1-identity",
            toolchain_scope: "rustc-version-and-target-scoped",
            required_fields: vec![
                "schema",
                "schema_version",
                "function_object_hash",
                "source_path",
                "function_name",
                "rustc_version",
                "target_triple",
                "extractor",
                "observations",
                "observation_hash",
                "extraction_caveats",
                "boundary",
            ],
            unsupported_reason: "bounded pilot source-token HIR fixture plus selected rustc HIR fixture shapes [checked-arithmetic, branch-control-flow] only; non-fixture HIR requests remain unsupported-planned until general extractor evidence is added",
            required_evidence_before_support: vec![
                "positive-cli-supplement-emit-check-smoke",
                "positive-rustc-hir-fixture-spike-emit-check-smoke",
                "positive-rustc-hir-selected-fixture-shape-smoke",
                "negative-context-mismatch-check-smoke",
                "negative-rustc-hir-fixture-spike-replay-mismatch-smoke",
                "negative-rustc-hir-selected-fixture-shape-mismatch-smoke",
                "non-proof-boundary-assertions",
            ],
            extractor_identity_expectations: vec![
                "extractor.name=octet-hir-pilot-source-token for default pilot scope",
                "extractor.name=octet-rustc-hir-fixture-spike for --spike rustc-hir-fixture",
                "fixture_shape=checked-arithmetic maps to checked.rs::checked",
                "fixture_shape=branch-control-flow maps to clamp.rs::clamp_to_limit",
                "extractor.version=cargo-octet-package-version",
                "rustc_version=runtime-rustc-vv",
                "target_triple=runtime-rustc-host",
            ],
            selected_fixture_shapes: vec!["checked-arithmetic", "branch-control-flow"],
            fixture_replay_command: "cargo octet object supplement emit --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --output-format json && cargo octet object supplement check --family hir --spike rustc-hir-fixture --fixture-shape checked-arithmetic --receipt RECEIPT.json --output-format json; repeat with --fixture-shape branch-control-flow",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            claims_cross_toolchain_stability: false,
        },
        ObjectSupplementalEvidenceContract {
            family: "thir",
            schema: THIR_SUPPLEMENT_SCHEMA,
            schema_version: SUPPLEMENT_SCHEMA_VERSION,
            status: ObjectSupplementStatus::UnsupportedPlanned.as_str(),
            relationship_to_v1: "optional-supplement-preserves-function-object-v1-identity",
            toolchain_scope: "rustc-version-target-and-thir-context-scoped",
            required_fields: vec![
                "schema",
                "schema_version",
                "function_object_hash",
                "rustc_version",
                "target_triple",
                "thir_context",
                "thir_observation_hash",
                "extraction_caveats",
                "boundary",
            ],
            unsupported_reason: "Octet has not implemented a THIR extractor or deterministic THIR supplement fixtures.",
            required_evidence_before_support: vec![
                "deterministic-thir-supplement-fixture",
                "toolchain-and-thir-context-scoped-replay-check",
                "non-proof-boundary-assertions",
            ],
            extractor_identity_expectations: vec![
                "deterministic-thir-extractor-name",
                "deterministic-thir-extractor-version",
                "rustc_version=exact-toolchain",
                "target_triple=exact-target",
            ],
            selected_fixture_shapes: Vec::new(),
            fixture_replay_command: "unsupported-planned: add deterministic THIR fixture emit/check evidence before enabling",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            claims_cross_toolchain_stability: false,
        },
        ObjectSupplementalEvidenceContract {
            family: "mir",
            schema: MIR_SUPPLEMENT_SCHEMA,
            schema_version: SUPPLEMENT_SCHEMA_VERSION,
            status: ObjectSupplementStatus::PilotSupported.as_str(),
            relationship_to_v1: "optional-supplement-preserves-function-object-v1-identity",
            toolchain_scope: "rustc-version-target-and-mir-context-scoped",
            required_fields: vec![
                "schema",
                "schema_version",
                "function_object_hash",
                "source_path",
                "function_name",
                "rustc_version",
                "target_triple",
                "extractor",
                "observations",
                "observation_hash",
                "extraction_caveats",
                "boundary",
            ],
            unsupported_reason: "bounded pilot source-token MIR summary fixture only; non-fixture MIR requests remain unsupported-planned until extractor evidence is added",
            required_evidence_before_support: vec![
                "positive-cli-supplement-emit-check-smoke",
                "negative-context-mismatch-check-smoke",
                "non-proof-boundary-assertions",
            ],
            extractor_identity_expectations: vec![
                "extractor.name=octet-mir-pilot-source-token",
                "extractor.version=cargo-octet-package-version",
                "rustc_version=runtime-rustc-vv",
                "target_triple=runtime-rustc-host",
            ],
            selected_fixture_shapes: Vec::new(),
            fixture_replay_command: "cargo octet object supplement emit --family mir --function checked src/lib.rs --output-format json && cargo octet object supplement check --family mir --receipt RECEIPT.json --function checked src/lib.rs --output-format json",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            claims_cross_toolchain_stability: false,
        },
    ]
}

fn print_corpus_contract_human(report: &ObjectCorpusContractReport) {
    println!("object corpus contract schema={} version={}", report.schema, report.schema_version);
    println!("source_of_truth={}", report.source_of_truth);
    println!("receipt schema={} version={}", report.receipt.schema, report.receipt.schema_version);
    println!("receipt required_fields={}", report.receipt.required_fields.join(","));
    println!("receipt compared_fields={}", report.receipt.compared_fields.join(","));
    println!("check_report schema={} version={}", report.check_report.schema, report.check_report.schema_version);
    println!("check_report required_fields={}", report.check_report.required_fields.join(","));
    println!("check_report status_values={}", report.check_report.status_values.join(","));
    for supplement in &report.supplemental_evidence {
        println!(
            "supplemental_evidence family={} schema={} version={} status={} relationship_to_v1={} toolchain_scope={}",
            supplement.family,
            supplement.schema,
            supplement.schema_version,
            supplement.status,
            supplement.relationship_to_v1,
            supplement.toolchain_scope
        );
        println!(
            "supplemental_evidence {} required_fields={}",
            supplement.family,
            supplement.required_fields.join(",")
        );
        println!("supplemental_evidence {} unsupported_reason={}", supplement.family, supplement.unsupported_reason);
        println!(
            "supplemental_evidence {} extractor_identity_expectations={}",
            supplement.family,
            supplement.extractor_identity_expectations.join(",")
        );
        println!(
            "supplemental_evidence {} selected_fixture_shapes={}",
            supplement.family,
            supplement.selected_fixture_shapes.join(",")
        );
        println!(
            "supplemental_evidence {} fixture_replay_command={}",
            supplement.family, supplement.fixture_replay_command
        );
        println!(
            "supplemental_evidence {} claims_correctness={} claims_semantic_equivalence={} claims_cross_toolchain_stability={}",
            supplement.family,
            supplement.claims_correctness,
            supplement.claims_semantic_equivalence,
            supplement.claims_cross_toolchain_stability
        );
    }
    println!("boundary claims_correctness={}", report.boundaries.claims_correctness);
    println!("boundary claims_semantic_equivalence={}", report.boundaries.claims_semantic_equivalence);
}

fn print_corpus_check_report(report: &ObjectCorpusCheckReport, output_format: OutputFormat) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => print_corpus_check_human(report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => {
            return Err("object corpus check does not support SARIF output; use human or json".to_owned());
        }
    }
    Ok(())
}

fn corpus_receipt(paths: &[PathBuf], objects: &[FunctionObjectV1]) -> ObjectCorpusReceipt {
    let source_paths = objects
        .iter()
        .map(|object| object.source_path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut object_hashes = objects.iter().map(|object| object.object_hash.as_str()).collect::<Vec<_>>();
    object_hashes.sort_unstable();
    let object_set_hash = determinism::full_hash(&object_hashes.join("\n"));

    ObjectCorpusReceipt {
        schema: CORPUS_RECEIPT_SCHEMA,
        schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
        mode: "corpus-receipt",
        object_count: objects.len(),
        source_paths,
        object_set_hash,
        caveat_counts: corpus_caveat_counts(objects),
        pure_cache_blocked_count: objects.iter().filter(|object| !object.pure_cache_eligibility.eligible).count(),
        replay: CorpusReplay {
            command: corpus_replay_command(paths),
            tool_name: "cargo-octet",
            tool_version: env!("CARGO_PKG_VERSION"),
            determinism_note: "Receipt bytes are deterministic for the same selected source inputs and cargo-octet version; this is not a correctness proof.",
        },
        boundary: CorpusBoundary {
            identity_scope: "review-and-provenance-evidence",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            detail: "Function object corpus receipts summarize source-token object identity and caveats; they do not prove behavioral correctness, semantic equivalence, dependency resolution, macro expansion identity, or pure-cache eligibility.",
        },
    }
}

fn corpus_caveat_taxonomy_report(objects: &[FunctionObjectV1]) -> ObjectCorpusCaveatTaxonomyReport {
    let source_paths = objects
        .iter()
        .map(|object| object.source_path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut source_buckets = Vec::new();
    for source_path in &source_paths {
        let bucket_objects =
            objects.iter().filter(|object| &object.source_path == source_path).cloned().collect::<Vec<_>>();
        let blocker_names = bucket_objects
            .iter()
            .flat_map(|object| object.pure_cache_eligibility.blockers.iter().map(|blocker| (*blocker).to_owned()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        source_buckets.push(ObjectCorpusCaveatSourceBucket {
            source_path: source_path.clone(),
            object_count: bucket_objects.len(),
            caveat_counts: corpus_caveat_counts(&bucket_objects),
            pure_cache_blocked_count: bucket_objects
                .iter()
                .filter(|object| !object.pure_cache_eligibility.eligible)
                .count(),
            blocker_names,
        });
    }

    ObjectCorpusCaveatTaxonomyReport {
        schema: "octet.function-object-corpus-caveat-taxonomy.v1",
        schema_version: CORPUS_RECEIPT_SCHEMA_VERSION,
        mode: "corpus-caveat-taxonomy",
        object_count: objects.len(),
        source_paths,
        source_buckets,
        caveat_counts: corpus_caveat_counts(objects),
        pure_cache_blocked_count: objects.iter().filter(|object| !object.pure_cache_eligibility.eligible).count(),
        boundary: CorpusBoundary {
            identity_scope: "review-and-provenance-evidence",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            detail: "Function object corpus caveat taxonomy reports explain source-token caveat and pure-cache blocker distribution; they do not prove behavioral correctness, semantic equivalence, dependency resolution, macro expansion identity, or pure-cache eligibility.",
        },
    }
}

fn print_corpus_caveat_taxonomy_human(report: &ObjectCorpusCaveatTaxonomyReport) {
    println!("object corpus caveat taxonomy: {}", report.schema);
    println!("object_count={}", report.object_count);
    println!("pure_cache_blocked_count={}", report.pure_cache_blocked_count);
    for bucket in &report.source_buckets {
        println!(
            "source_bucket path={} objects={} pure_cache_blocked={} blockers={}",
            bucket.source_path,
            bucket.object_count,
            bucket.pure_cache_blocked_count,
            if bucket.blocker_names.is_empty() {
                "none".to_owned()
            } else {
                bucket.blocker_names.join(",")
            }
        );
    }
    println!("boundary claims_correctness={}", report.boundary.claims_correctness);
    println!("boundary claims_semantic_equivalence={}", report.boundary.claims_semantic_equivalence);
}

fn corpus_caveat_counts(objects: &[FunctionObjectV1]) -> ObjectCorpusCaveatCounts {
    let mut dependency_status = BTreeMap::new();
    let mut effect_kind = BTreeMap::new();
    let mut macro_status = BTreeMap::new();
    let mut pure_cache_blocker = BTreeMap::new();

    for object in objects {
        for caveat in &object.dependency_caveats {
            increment(&mut dependency_status, caveat.status);
        }
        for caveat in &object.effect_caveats {
            increment(&mut effect_kind, caveat.kind);
        }
        for caveat in &object.macro_caveats {
            increment(&mut macro_status, caveat.status);
        }
        for blocker in &object.pure_cache_eligibility.blockers {
            increment(&mut pure_cache_blocker, blocker);
        }
    }

    ObjectCorpusCaveatCounts {
        dependency_status,
        effect_kind,
        macro_status,
        pure_cache_blocker,
    }
}

fn increment(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_owned()).or_insert(0) += 1;
}

fn corpus_replay_command(paths: &[PathBuf]) -> String {
    let selected = if paths.is_empty() {
        vec![".".to_owned()]
    } else {
        paths.iter().map(|path| path.to_string_lossy().into_owned()).collect::<Vec<_>>()
    };
    format!("cargo octet object corpus receipt --output RECEIPT.json {}", selected.join(" "))
}

fn run_marker(args: ObjectArgs, update: bool) -> Result<i32, String> {
    let mut files = collect_files(&args.paths)?;
    let mut records = Vec::new();
    for path in files.drain(..) {
        let source = fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let mut replacements = Vec::new();
        for span in fingerprint::find_functions(&source) {
            if let Some(selected) = args.function.as_deref()
                && selected != span.name
            {
                continue;
            }
            let function_source = &source[span.start..span.end];
            let object = function_object_for_source(&path, &source, &span, function_source)?;
            let marker = object_marker_before(&source, span.start);
            let status = match marker.as_ref().map(|marker| marker.stored_hash.as_str()) {
                Some(stored) if stored == object.object_hash => "match",
                Some(_) => "mismatch",
                None => "missing",
            };
            if update && status != "match" {
                let review = marker.as_ref().and_then(|m| m.review.as_deref()).unwrap_or("TODO");
                let evidence = marker.as_ref().and_then(|m| m.evidence.as_deref()).unwrap_or("TODO");
                let line = format!("// octet-object: {}; review={review}; evidence={evidence}\n", object.object_hash);
                if let Some(marker) = marker.as_ref() {
                    replacements.push((marker.start, marker.end, line));
                } else {
                    replacements.push((span.start, span.start, line));
                }
            }
            records.push(ObjectMarkerRecord {
                path: normalized_source_path(&path),
                name: object.name,
                stored_hash: marker.as_ref().map(|marker| marker.stored_hash.clone()),
                computed_hash: object.object_hash,
                status: if update && status != "match" { "written" } else { status },
                review: marker.as_ref().and_then(|marker| marker.review.clone()),
                evidence: marker.as_ref().and_then(|marker| marker.evidence.clone()),
            });
        }
        if !replacements.is_empty() {
            let mut updated = source;
            for (start, end, replacement) in replacements.into_iter().rev() {
                updated.replace_range(start..end, &replacement);
            }
            fs::write(&path, updated).map_err(|error| format!("write {}: {error}", path.display()))?;
        }
    }
    if records.is_empty() {
        return Err("no functions matched object marker selection".to_owned());
    }
    let report = ObjectMarkerReport {
        schema_version: 1,
        mode: if update { "update" } else { "check" },
        records,
    };
    match args.output_format {
        OutputFormat::Human => print_marker_human(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => return Err("object marker does not support SARIF output; use human or json".to_owned()),
    }
    let failing = !update
        && report
            .records
            .iter()
            .any(|record| record.status != "match" || record.review.is_none() || record.evidence.is_none());
    Ok(if failing { 1 } else { 0 })
}

fn collect_objects(paths: &[PathBuf], function: Option<&str>) -> Result<Vec<ObjectRecord>, String> {
    let mut objects = Vec::new();
    for path in collect_files(paths)? {
        let source = fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        for span in fingerprint::find_functions(&source) {
            if let Some(selected) = function
                && selected != span.name
            {
                continue;
            }
            let function_source = &source[span.start..span.end];
            let object = function_object_for_source(&path, &source, &span, function_source)?;
            objects.push(ObjectRecord { object });
        }
    }
    Ok(objects)
}

fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let inputs = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    for path in inputs {
        fingerprint::collect_rust_files(&path, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn function_object_for_source(
    path: &Path,
    full_source: &str,
    span: &fingerprint::FunctionSpan,
    function_source: &str,
) -> Result<FunctionObjectV1, String> {
    let normalized = fingerprint::normalize_tokens(function_source);
    let normalized_body_hash = determinism::full_hash(&normalized);
    let body_hash = normalized_body_hash.clone();
    let signature_hash = signature_hash(function_source);
    let dependency_set_hash = fingerprint::dependency_set_hash(function_source);
    let dependency_status = fingerprint::dependency_identity_status(function_source);
    let dependency_caveats = dependency_caveats(dependency_status);
    let effect_caveats = effect_caveats(function_source);
    let macro_caveats = macro_caveats(span.macro_calls);
    let caveat_summary = caveat_summary(&dependency_caveats, &effect_caveats, &macro_caveats);
    let pure_cache_eligibility = pure_cache_eligibility(&dependency_caveats, &effect_caveats, &macro_caveats);
    let semantic_id = determinism::short_hash(&format!(
        "schema={FUNCTION_OBJECT_SCHEMA}\nname={}\nbody={normalized_body_hash}\nsignature={signature_hash}\ndeps={dependency_set_hash}\ndep_status={dependency_status}\nnormalization={NORMALIZATION_ALGORITHM}\ntool={}\n",
        span.name,
        env!("CARGO_PKG_VERSION"),
    ));
    let source_span = source_span(full_source, span.start, span.end)?;
    let mut object = FunctionObjectV1 {
        schema: FUNCTION_OBJECT_SCHEMA,
        object_hash: String::new(),
        semantic_id,
        name: span.name.clone(),
        source_path: normalized_source_path(path),
        source_span,
        body_hash,
        normalized_body_hash,
        signature_hash,
        dependency_set_hash,
        dependency_caveats,
        effect_caveats,
        macro_caveats,
        normalization: NormalizationMetadata {
            algorithm: NORMALIZATION_ALGORITHM,
            tool_name: "cargo-octet",
            tool_version: env!("CARGO_PKG_VERSION"),
            hash_algorithm: "BLAKE3",
        },
        caveat_summary,
        pure_cache_eligibility,
        correctness_claim: CorrectnessClaim {
            claims_correctness: false,
            detail: "Function object hashes identify reviewed source evidence and caveats; they do not prove behavioral correctness.",
        },
    };
    object.object_hash = object_hash(&object)?;
    Ok(object)
}

fn object_hash(object: &FunctionObjectV1) -> Result<String, String> {
    let payload = serde_json::json!({
        "schema": object.schema,
        "semantic_id": object.semantic_id,
        "name": object.name,
        "body_hash": object.body_hash,
        "normalized_body_hash": object.normalized_body_hash,
        "signature_hash": object.signature_hash,
        "dependency_set_hash": object.dependency_set_hash,
        "dependency_caveats": object.dependency_caveats,
        "effect_caveats": object.effect_caveats,
        "macro_caveats": object.macro_caveats,
        "normalization": object.normalization,
        "caveat_summary": object.caveat_summary,
        "pure_cache_eligibility": object.pure_cache_eligibility,
        "correctness_claim": object.correctness_claim,
    });
    let bytes = serde_json::to_string(&payload).map_err(|error| format!("canonicalize function object: {error}"))?;
    Ok(determinism::full_hash(&bytes))
}

fn signature_hash(function_source: &str) -> String {
    let signature = function_source.split('{').next().unwrap_or(function_source);
    determinism::full_hash(&fingerprint::normalize_tokens(signature))
}

fn dependency_caveats(status: &'static str) -> Vec<DependencyCaveat> {
    let detail = match status {
        DEPENDENCY_STATUS_MACRO_CONTEXT => {
            "Dependency identity is token-derived and the function contains macro calls; dependencies are not resolved through rustc/HIR."
        }
        _ => "Dependency identity is a source-scanner token set, not a resolved dependency graph.",
    };
    vec![DependencyCaveat {
        kind: "dependency_identity",
        status,
        detail,
    }]
}

fn effect_caveats(function_source: &str) -> Vec<EffectCaveat> {
    let mut caveats = Vec::new();
    let normalized = fingerprint::normalize_tokens(function_source);
    for (needle, kind, detail) in [
        ("unsafe", "unsafe", "Source tokens contain unsafe; effect summary remains caveated."),
        ("panic", "panic", "Source tokens contain panic-like spelling; effect summary remains caveated."),
        ("unwrap", "panic", "Source tokens contain unwrap-like spelling; effect summary remains caveated."),
        ("expect", "panic", "Source tokens contain expect-like spelling; effect summary remains caveated."),
        (
            "fs",
            "filesystem",
            "Source tokens contain filesystem-like spelling; effect summary remains caveated.",
        ),
        ("net", "network", "Source tokens contain network-like spelling; effect summary remains caveated."),
        (
            "Command",
            "process",
            "Source tokens contain process-spawn-like spelling; effect summary remains caveated.",
        ),
        (
            "SystemTime",
            "time",
            "Source tokens contain wall-clock-like spelling; effect summary remains caveated.",
        ),
    ] {
        if normalized.split_whitespace().any(|token| token == needle) {
            caveats.push(EffectCaveat {
                kind,
                status: "source-token-caveat",
                detail,
            });
        }
    }
    if caveats.is_empty() {
        caveats.push(EffectCaveat {
            kind: "effect_summary",
            status: "unknown-source-scanner-only",
            detail: "No coarse ambient effect token was found, but v1 source scanning does not prove purity.",
        });
    }
    caveats
}

fn macro_caveats(macro_calls: bool) -> Vec<MacroCaveat> {
    if macro_calls {
        vec![MacroCaveat {
            kind: "macro_call",
            status: "review-required",
            detail: "Function body contains a macro call; v1 object identity is based on source tokens, not expanded macro output.",
        }]
    } else {
        vec![MacroCaveat {
            kind: "macro_call",
            status: "none-detected",
            detail: "No macro call was detected by the v1 source scanner.",
        }]
    }
}

fn caveat_summary(
    dependency_caveats: &[DependencyCaveat],
    effect_caveats: &[EffectCaveat],
    macro_caveats: &[MacroCaveat],
) -> CaveatSummary {
    let dependency_identity = if dependency_caveats.iter().any(|caveat| caveat.status == "resolved") {
        "resolved"
    } else {
        "unresolved-source-scanner"
    };
    let macro_expansion = if macro_caveats.iter().any(|caveat| caveat.status == "review-required") {
        "macro-expansion-not-modeled"
    } else {
        "none-detected"
    };
    let effect_summary = if effect_caveats.iter().any(|caveat| caveat.status == "source-token-caveat") {
        "effect-caveated"
    } else {
        "unknown-not-pure-proof"
    };
    CaveatSummary {
        dependency_identity,
        macro_expansion,
        effect_summary,
        cache_policy: "pure-cache-blocked-unless-external-policy-accepts-caveats",
    }
}

fn pure_cache_eligibility(
    dependency_caveats: &[DependencyCaveat],
    effect_caveats: &[EffectCaveat],
    macro_caveats: &[MacroCaveat],
) -> PureCacheEligibility {
    let mut blockers = Vec::new();
    if dependency_caveats.iter().any(|caveat| caveat.status != "resolved") {
        blockers.push("dependency-identity-unresolved");
    }
    if effect_caveats.iter().any(|caveat| caveat.status != "resolved-pure") {
        blockers.push("effect-summary-unresolved");
    }
    if macro_caveats.iter().any(|caveat| caveat.status == "review-required") {
        blockers.push("macro-expansion-unmodeled");
    }
    PureCacheEligibility {
        eligible: blockers.is_empty(),
        status: if blockers.is_empty() {
            "eligible"
        } else {
            "blocked-by-caveats"
        },
        blockers,
    }
}

fn source_span(source: &str, start: usize, end: usize) -> Result<SourceSpan, String> {
    let start_line = line_number(source, start)?;
    let end_line = line_number(source, end.saturating_sub(1))?;
    Ok(SourceSpan { start_line, end_line })
}

fn line_number(source: &str, byte_index: usize) -> Result<u32, String> {
    if byte_index > source.len() {
        return Err("function span exceeded source length".to_owned());
    }
    let line_count = source[..byte_index].bytes().filter(|byte| *byte == b'\n').count() + 1;
    u32::try_from(line_count).map_err(|_| "source line count exceeds u32".to_owned())
}

fn normalized_source_path(path: &Path) -> String {
    let relative = std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok().map(PathBuf::from))
        .unwrap_or_else(|| path.to_path_buf());
    determinism::normalize_path(&relative)
}

fn object_marker_before(source: &str, item_start: usize) -> Option<ObjectMarker> {
    let prefix = &source[..item_start];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let previous_end = line_start.saturating_sub(1);
    let previous_start = source[..previous_end].rfind('\n').map_or(0, |index| index + 1);
    let previous = source[previous_start..previous_end].trim();
    if !previous.starts_with(OBJECT_MARKER) {
        return None;
    }
    let hash_start = previous.find(HASH_PREFIX)? + HASH_PREFIX.len();
    let stored_hash =
        format!("b3:{}", previous[hash_start..].chars().take_while(|c| c.is_ascii_hexdigit()).collect::<String>());
    if stored_hash == "b3:" {
        return None;
    }
    Some(ObjectMarker {
        start: previous_start,
        end: if previous_end < source.len() {
            previous_end + 1
        } else {
            previous_end
        },
        stored_hash,
        review: marker_value(previous, "review"),
        evidence: marker_value(previous, "evidence"),
    })
}

fn marker_value(line: &str, key: &str) -> Option<String> {
    let value_start = line.find(&format!("{key}="))? + key.len() + 1;
    let value = line[value_start..]
        .split([';', ',', ')', ']'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_owned();
    (!value.is_empty() && value != "TODO").then_some(value)
}

fn run_diff(
    old_path: &Path,
    new_path: &Path,
    pointer_manifest: Option<&Path>,
    output_format: OutputFormat,
) -> Result<i32, String> {
    let old = read_single_object(old_path)?;
    let new = read_single_object(new_path)?;
    let pointer_changes = if let Some(path) = pointer_manifest {
        pointer_changes(path, string_field(&old, "object_hash")?, string_field(&new, "object_hash")?)?
    } else {
        Vec::new()
    };
    let report = serde_json::json!({
        "schema_version": 1,
        "mode": "diff",
        "semantic_name": string_field(&new, "name").unwrap_or_else(|_| "unknown".to_owned()),
        "old_object_hash": string_field(&old, "object_hash")?,
        "new_object_hash": string_field(&new, "object_hash")?,
        "body_changed": old.get("body_hash") != new.get("body_hash"),
        "signature_changed": old.get("signature_hash") != new.get("signature_hash"),
        "dependency_caveats_changed": old.get("dependency_caveats") != new.get("dependency_caveats"),
        "effect_caveats_changed": old.get("effect_caveats") != new.get("effect_caveats"),
        "macro_caveats_changed": old.get("macro_caveats") != new.get("macro_caveats"),
        "pointer_changes": pointer_changes,
    });
    match output_format {
        OutputFormat::Human => {
            println!("octet object diff");
            println!("body_changed={}", report["body_changed"]);
            println!("signature_changed={}", report["signature_changed"]);
            println!("dependency_caveats_changed={}", report["dependency_caveats_changed"]);
            println!("effect_caveats_changed={}", report["effect_caveats_changed"]);
            println!("macro_caveats_changed={}", report["macro_caveats_changed"]);
            println!("pointer_changes={}", report["pointer_changes"].as_array().map_or(0, Vec::len));
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => return Err("object diff does not support SARIF output; use human or json".to_owned()),
    }
    Ok(0)
}

fn run_pointer_set(
    manifest_path: &Path,
    name: &str,
    hash: &str,
    objects_path: &Path,
    receipt_path: &Path,
) -> Result<i32, String> {
    let available = object_hashes(objects_path)?;
    if !available.contains(hash) {
        return Err(format!("pointer hash `{hash}` does not resolve to an object record"));
    }
    let mut manifest = read_pointer_manifest(manifest_path)?;
    manifest.schema_version = 1;
    let old_hash = manifest.pointers.get(name).map(|pointer| pointer.object_hash.clone());
    let diff_summary = match old_hash.as_deref() {
        Some(old) if old != hash => format!("pointer-moved:{old}->{hash}"),
        Some(_) => "pointer-unchanged".to_owned(),
        None => "pointer-created".to_owned(),
    };
    let rollback_handle = format!("{}#{}", manifest_path.display(), name);
    if let Some(old) = old_hash.as_ref()
        && old != hash
    {
        manifest.history.entry(name.to_owned()).or_default().push(NamePointerHistory {
            old_hash: old.clone(),
            new_hash: hash.to_owned(),
            semantic_name: name.to_owned(),
            diff_summary: diff_summary.clone(),
            rollback_handle: rollback_handle.clone(),
        });
    }
    manifest.pointers.insert(name.to_owned(), NamePointer {
        object_hash: hash.to_owned(),
        semantic_name: name.to_owned(),
    });
    write_json(manifest_path, &manifest)?;
    let receipt = PointerReceipt {
        schema_version: 1,
        semantic_name: name.to_owned(),
        old_hash,
        new_hash: hash.to_owned(),
        diff_summary,
        rollback_handle,
    };
    write_json(receipt_path, &receipt)?;
    println!("{}", serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?);
    Ok(0)
}

fn run_pointer_validate(manifest_path: &Path, objects_path: &Path) -> Result<i32, String> {
    let manifest = read_pointer_manifest(manifest_path)?;
    let available = object_hashes(objects_path)?;
    let missing = manifest
        .pointers
        .iter()
        .filter_map(|(name, pointer)| (!available.contains(&pointer.object_hash)).then_some(name.clone()))
        .collect::<Vec<_>>();
    let report = serde_json::json!({
        "schema_version": 1,
        "mode": "pointer-validate",
        "status": if missing.is_empty() { "resolved" } else { "missing-object" },
        "pointer_count": manifest.pointers.len(),
        "missing_pointers": missing,
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?);
    Ok(if report["status"] == "resolved" { 0 } else { 1 })
}

fn run_bundle_export(
    objects_path: &Path,
    pointers_path: Option<&Path>,
    output_path: &Path,
    replay_command: &str,
) -> Result<i32, String> {
    let objects_value = read_json(objects_path)?;
    let mut objects = objects_value
        .get("objects")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .ok_or_else(|| format!("{} must contain an `objects` array", objects_path.display()))?;
    objects.sort_by(|left, right| {
        string_value(left, "object_hash")
            .unwrap_or_default()
            .cmp(&string_value(right, "object_hash").unwrap_or_default())
    });
    let pointers = if let Some(path) = pointers_path {
        let manifest = read_pointer_manifest(path)?;
        let available = objects
            .iter()
            .filter_map(|object| object.get("object_hash").and_then(serde_json::Value::as_str).map(str::to_owned))
            .collect::<BTreeSet<_>>();
        for (name, pointer) in &manifest.pointers {
            if !available.contains(&pointer.object_hash) {
                return Err(format!("bundle pointer `{name}` references missing object `{}`", pointer.object_hash));
            }
        }
        Some(serde_json::to_value(manifest).map_err(|error| format!("serialize pointer manifest: {error}"))?)
    } else {
        None
    };
    let replay = BundleReplay {
        command: replay_command.to_owned(),
        tool_name: "cargo-octet",
        tool_version: env!("CARGO_PKG_VERSION"),
        determinism_note: "Replay command is metadata for rerunning object emission/checks; bundle hashes identify evidence bytes, not correctness.",
    };
    let manifest_payload = serde_json::json!({
        "schema": "octet.function-object-bundle.v1",
        "schema_version": 1,
        "object_hashes": objects.iter().filter_map(|object| object.get("object_hash").and_then(serde_json::Value::as_str)).collect::<Vec<_>>(),
        "pointer_names": pointers.as_ref().and_then(|value| value.get("pointers")).and_then(serde_json::Value::as_object).map(|map| map.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
        "replay_command": replay.command,
    });
    let manifest_hash = determinism::full_hash(
        &serde_json::to_string(&manifest_payload).map_err(|error| format!("serialize bundle manifest: {error}"))?,
    );
    let mut bundle = ObjectBundle {
        schema: "octet.function-object-bundle.v1",
        schema_version: 1,
        bundle_hash: String::new(),
        manifest_hash,
        object_count: objects.len(),
        pointer_count: pointers
            .as_ref()
            .and_then(|value| value.get("pointers"))
            .and_then(serde_json::Value::as_object)
            .map_or(0, serde_json::Map::len),
        replay,
        objects,
        pointers,
    };
    let bundle_payload = serde_json::json!({
        "schema": bundle.schema,
        "schema_version": bundle.schema_version,
        "manifest_hash": bundle.manifest_hash,
        "object_count": bundle.object_count,
        "pointer_count": bundle.pointer_count,
        "replay": bundle.replay,
        "objects": bundle.objects,
        "pointers": bundle.pointers,
    });
    bundle.bundle_hash = determinism::full_hash(
        &serde_json::to_string(&bundle_payload).map_err(|error| format!("serialize bundle payload: {error}"))?,
    );
    write_json(output_path, &bundle)?;
    println!("{}", serde_json::to_string_pretty(&bundle).map_err(|error| error.to_string())?);
    Ok(0)
}

fn read_single_object(path: &Path) -> Result<serde_json::Value, String> {
    let value = read_json(path)?;
    if let Some(objects) = value.get("objects").and_then(serde_json::Value::as_array) {
        return objects.first().cloned().ok_or_else(|| format!("{} has no objects", path.display()));
    }
    Ok(value)
}

fn object_hashes(path: &Path) -> Result<BTreeSet<String>, String> {
    let value = read_json(path)?;
    let objects = value.get("objects").and_then(serde_json::Value::as_array).ok_or_else(|| {
        format!(
            "{} must contain an `objects` array from `cargo octet object emit --output-format json`",
            path.display()
        )
    })?;
    Ok(objects
        .iter()
        .filter_map(|object| object.get("object_hash").and_then(serde_json::Value::as_str).map(str::to_owned))
        .collect())
}

fn pointer_changes(path: &Path, old_hash: String, new_hash: String) -> Result<Vec<serde_json::Value>, String> {
    let manifest = read_pointer_manifest(path)?;
    Ok(manifest
        .pointers
        .iter()
        .filter_map(|(name, pointer)| {
            (pointer.object_hash == old_hash || pointer.object_hash == new_hash).then(|| {
                serde_json::json!({
                    "semantic_name": name,
                    "old_hash": old_hash,
                    "new_hash": new_hash,
                    "target_matches_new": pointer.object_hash == new_hash,
                })
            })
        })
        .collect())
}

fn string_field(value: &serde_json::Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("object missing `{field}`"))
}

fn string_value(value: &serde_json::Value, field: &str) -> Option<String> {
    value.get(field).and_then(serde_json::Value::as_str).map(str::to_owned)
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn read_pointer_manifest(path: &Path) -> Result<NamePointerManifest, String> {
    if !path.exists() {
        return Ok(NamePointerManifest::default());
    }
    serde_json::from_value(read_json(path)?).map_err(|error| format!("parse pointer manifest: {error}"))
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|error| format!("serialize json: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn print_corpus_check_human(report: &ObjectCorpusCheckReport) {
    println!("octet object corpus check status={}", report.status);
    println!("receipt_path={}", report.receipt_path);
    for error in &report.errors {
        println!(
            "error kind={} field={} message={}",
            error.kind,
            error.field.as_deref().unwrap_or("none"),
            error.message
        );
    }
    for comparison in &report.comparisons {
        println!("{}={}", comparison.field, comparison.status);
    }
}

fn print_marker_human(report: &ObjectMarkerReport) {
    println!("octet object {}", report.mode);
    for record in &report.records {
        println!(
            "{} {} status={} stored_hash={} computed_hash={} review={} evidence={}",
            record.path,
            record.name,
            record.status,
            record.stored_hash.as_deref().unwrap_or("none"),
            record.computed_hash,
            record.review.as_deref().unwrap_or("missing"),
            record.evidence.as_deref().unwrap_or("missing"),
        );
    }
    println!("summary total={}", report.records.len());
}

fn print_human(report: &FunctionObjectReport) {
    println!("octet object {}", report.mode);
    for object in &report.objects {
        println!(
            "{} {} object_hash={} semantic_id={} body_hash={} dependency_caveats={} effect_caveats={} correctness_claims={}",
            object.source_path,
            object.name,
            object.object_hash,
            object.semantic_id,
            object.body_hash,
            object.dependency_caveats.len(),
            object.effect_caveats.len(),
            object.correctness_claim.claims_correctness,
        );
    }
    println!("summary total={}", report.objects.len());
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ObjectSupplementStatus;
    use super::PilotSupplementContext;
    use super::RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH;
    use super::RUSTC_HIR_SPIKE_BRANCH_FUNCTION;
    use super::RUSTC_HIR_SPIKE_EXTRACTOR_NAME;
    use super::RUSTC_HIR_SPIKE_EXTRACTOR_VERSION;
    use super::RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH;
    use super::RUSTC_HIR_SPIKE_FUNCTION;
    use super::SUPPLEMENT_STATUS_VALUES;
    use super::SupplementFamilyArg;
    use super::canonical_supplement_json;
    use super::effect_caveats;
    use super::function_object_for_source;
    use super::pilot_hir_supplement;
    use super::pilot_mir_supplement;
    use super::rustc_hir_spike_supplement;
    use super::supplement_comparisons;
    use crate::fingerprint;

    #[test]
    fn function_object_rejects_correctness_claims() {
        let source = "pub fn checked(input: u32) -> u32 { input + 1 }\n";
        let spans = fingerprint::find_functions(source);
        let object = function_object_for_source(Path::new("src/lib.rs"), source, &spans[0], source)
            .expect("build function object");
        assert!(!object.object_hash.is_empty());
        assert!(!object.correctness_claim.claims_correctness);
        assert!(object.correctness_claim.detail.contains("do not prove"));
    }

    #[test]
    fn function_object_hash_is_stable_for_comments_and_formatting() {
        let one = "pub fn checked(input: u32) -> u32 { // comment\n input + 1 }\n";
        let two = "pub fn checked( input:u32)->u32{input+1}\n";
        let one_span = fingerprint::find_functions(one);
        let two_span = fingerprint::find_functions(two);
        let one_object = function_object_for_source(
            Path::new("src/lib.rs"),
            one,
            &one_span[0],
            &one[one_span[0].start..one_span[0].end],
        )
        .expect("one object");
        let two_object = function_object_for_source(
            Path::new("src/lib.rs"),
            two,
            &two_span[0],
            &two[two_span[0].start..two_span[0].end],
        )
        .expect("two object");
        assert_eq!(one_object.normalized_body_hash, two_object.normalized_body_hash);
        assert_eq!(one_object.object_hash, two_object.object_hash);
    }

    #[test]
    fn pilot_hir_supplement_attaches_to_v1_identity() {
        let source = "pub fn checked(input: u32) -> u32 { input + 1 }\n";
        let spans = fingerprint::find_functions(source);
        let object = function_object_for_source(Path::new("src/lib.rs"), source, &spans[0], source)
            .expect("build function object");
        let context = pilot_context();
        let supplement = pilot_hir_supplement(&object, &context);
        let json = canonical_supplement_json(&supplement).expect("canonical supplement json");

        assert_eq!(supplement.schema, "octet.function-object-hir-supplement.v1");
        assert_eq!(supplement.status, ObjectSupplementStatus::PilotSupported.as_str());
        assert_eq!(supplement.function_object_hash, object.object_hash);
        assert_eq!(supplement.source_path, "src/lib.rs");
        assert_eq!(supplement.function_name, "checked");
        assert_eq!(supplement.rustc_version, "rustc 1.92.0-nightly fixture");
        assert_eq!(supplement.extractor.name, "octet-hir-pilot-fixture");
        assert!(supplement.observations.contains_key("hir_context"));
        assert!(json.contains("attached-supplement-preserves-function-object-v1-identity"));
        assert!(json.contains("\"claims_correctness\": false"));
        assert_eq!(json, canonical_supplement_json(&supplement).expect("repeat canonical supplement json"));
    }

    #[test]
    fn pilot_mir_supplement_records_closed_unsupported_summary_families() {
        let source = "pub fn checked(input: u32) -> u32 { input + 1 }\n";
        let spans = fingerprint::find_functions(source);
        let object = function_object_for_source(Path::new("src/lib.rs"), source, &spans[0], source)
            .expect("build function object");
        let supplement = pilot_mir_supplement(&object, &pilot_context());

        assert_eq!(SUPPLEMENT_STATUS_VALUES, &[
            ObjectSupplementStatus::UnsupportedPlanned.as_str(),
            ObjectSupplementStatus::PilotSupported.as_str(),
            ObjectSupplementStatus::Unknown.as_str(),
            ObjectSupplementStatus::Unsupported.as_str(),
        ]);
        assert_eq!(supplement.schema, "octet.function-object-mir-supplement.v1");
        assert_eq!(supplement.status, ObjectSupplementStatus::PilotSupported.as_str());
        for family in ["call", "panic", "drop", "async", "unsafe", "allocation", "effect"] {
            let observation = supplement.observations.get(family).expect("summary family present");
            assert_eq!(observation.status, ObjectSupplementStatus::Unknown.as_str());
            assert!(observation.detail.contains("explicit unknown/unsupported"));
        }
        assert!(!supplement.boundary.claims_cross_toolchain_stability);
        assert!(!supplement.boundary.claims_semantic_equivalence);
    }

    #[test]
    fn rustc_hir_spike_helper_records_fixture_context_and_hashes() {
        let fixture = Path::new(RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH);
        let supplement =
            rustc_hir_spike_supplement(fixture, RUSTC_HIR_SPIKE_FUNCTION).expect("rustc HIR spike supplement");
        let json = canonical_supplement_json(&supplement).expect("canonical rustc HIR supplement json");

        assert_eq!(supplement.schema, "octet.function-object-hir-supplement.v1");
        assert_eq!(supplement.family, "hir");
        assert_eq!(supplement.source_path, RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH);
        assert_eq!(supplement.function_name, RUSTC_HIR_SPIKE_FUNCTION);
        assert_eq!(supplement.extractor.name, RUSTC_HIR_SPIKE_EXTRACTOR_NAME);
        assert_eq!(supplement.extractor.version, RUSTC_HIR_SPIKE_EXTRACTOR_VERSION);
        assert!(supplement.rustc_version.contains("rustc "));
        assert!(!supplement.target_triple.is_empty());
        assert!(supplement.observations.contains_key("rustc_hir_tree_hash"));
        assert!(supplement.observations.contains_key("fixture_context"));
        assert!(supplement.observation_hash.starts_with("b3:"));
        assert!(json.contains("fixture-bounded"));
        assert!(json.contains("\"claims_correctness\": false"));
        assert!(json.contains("\"claims_semantic_equivalence\": false"));
    }

    #[test]
    fn rustc_hir_spike_helper_accepts_second_branch_fixture_shape() {
        let fixture = Path::new(RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH);
        let supplement = rustc_hir_spike_supplement(fixture, RUSTC_HIR_SPIKE_BRANCH_FUNCTION)
            .expect("branch rustc HIR spike supplement");
        let fixture_context = supplement.observations.get("fixture_context").expect("fixture context observation");

        assert_eq!(supplement.source_path, RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH);
        assert_eq!(supplement.function_name, RUSTC_HIR_SPIKE_BRANCH_FUNCTION);
        assert!(fixture_context.detail.contains("fixture_shape=branch-control-flow"));
        assert!(supplement.observations.contains_key("rustc_hir_tree_hash"));
        assert!(supplement.observation_hash.starts_with("b3:"));
        assert!(!supplement.boundary.claims_correctness);
        assert!(!supplement.boundary.claims_semantic_equivalence);
    }

    #[test]
    fn rustc_hir_spike_helper_rejects_cross_fixture_function_pair() {
        let error = rustc_hir_spike_supplement(
            Path::new(RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH),
            RUSTC_HIR_SPIKE_FUNCTION,
        )
        .expect_err("wrong function for branch fixture should fail closed");
        assert!(error.contains("fixture-bounded"));
        assert!(error.contains(RUSTC_HIR_SPIKE_BRANCH_FUNCTION));
    }

    #[test]
    fn rustc_hir_spike_helper_rejects_non_fixture_context() {
        let error = rustc_hir_spike_supplement(Path::new("src/lib.rs"), RUSTC_HIR_SPIKE_FUNCTION)
            .expect_err("non-fixture path should fail closed");
        assert!(error.contains("fixture-bounded"));
    }

    fn pilot_context() -> PilotSupplementContext {
        PilotSupplementContext {
            rustc_version: "rustc 1.92.0-nightly fixture".to_owned(),
            target_triple: "x86_64-unknown-linux-gnu".to_owned(),
            extractor_name: "octet-hir-pilot-fixture",
            extractor_version: "fixture-v1",
        }
    }

    #[test]
    fn supplement_check_comparisons_fail_closed_on_context_mismatch() {
        let source = "pub fn checked(input: u32) -> u32 { input + 1 }\n";
        let spans = fingerprint::find_functions(source);
        let object = function_object_for_source(Path::new("src/lib.rs"), source, &spans[0], source)
            .expect("build function object");
        let current = pilot_hir_supplement(&object, &pilot_context());
        let mut expected = serde_json::to_value(&current).expect("serialize expected supplement");
        expected["rustc_version"] = serde_json::json!("different-rustc");

        let comparisons = supplement_comparisons(&expected, &current);
        let rustc = comparisons
            .iter()
            .find(|comparison| comparison.field == "rustc_version")
            .expect("rustc comparison present");
        let boundary = comparisons
            .iter()
            .find(|comparison| comparison.field == "boundary")
            .expect("boundary comparison present");

        assert_eq!(rustc.status, "mismatch");
        assert_eq!(boundary.status, "match");
        assert!(!current.boundary.claims_correctness);
        assert!(!current.boundary.claims_semantic_equivalence);
    }

    #[test]
    fn rustc_hir_spike_replay_compares_fixture_observations_and_caveats() {
        let fixture = Path::new(RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH);
        let current =
            rustc_hir_spike_supplement(fixture, RUSTC_HIR_SPIKE_FUNCTION).expect("rustc HIR spike supplement");
        let mut expected = serde_json::to_value(&current).expect("serialize expected supplement");
        expected["source_path"] = serde_json::json!("different/src.rs");
        expected["function_name"] = serde_json::json!("different_function");
        expected["extractor"]["name"] = serde_json::json!("different-extractor");
        expected["observations"]["rustc_hir_tree_hash"]["detail"] = serde_json::json!("tampered observation");
        expected["observation_hash"] = serde_json::json!("b3:tampered");
        expected["extraction_caveats"][0]["detail"] = serde_json::json!("tampered caveat");

        let comparisons = supplement_comparisons(&expected, &current);
        for field in [
            "source_path",
            "function_name",
            "extractor",
            "observations",
            "observation_hash",
            "extraction_caveats",
        ] {
            let comparison = comparisons
                .iter()
                .find(|comparison| comparison.field == field)
                .unwrap_or_else(|| panic!("missing comparison field {field}: {comparisons:#?}"));
            assert_eq!(comparison.status, "mismatch", "{field} should fail closed");
        }
        let boundary = comparisons
            .iter()
            .find(|comparison| comparison.field == "boundary")
            .expect("boundary comparison present");
        assert_eq!(boundary.status, "match");
        assert!(!current.boundary.claims_correctness);
        assert!(!current.boundary.claims_semantic_equivalence);
        assert!(!current.boundary.claims_pure_cache_eligibility);
        assert!(!current.boundary.claims_cross_toolchain_stability);
    }

    #[test]
    fn runtime_supplement_family_context_names_extractor() {
        let hir = super::runtime_pilot_context(SupplementFamilyArg::Hir);
        let mir = super::runtime_pilot_context(SupplementFamilyArg::Mir);
        assert_eq!(hir.extractor_name, "octet-hir-pilot-source-token");
        assert_eq!(mir.extractor_name, "octet-mir-pilot-source-token");
        assert!(!hir.extractor_version.is_empty());
    }

    #[test]
    fn unknown_effects_do_not_claim_purity() {
        let caveats = effect_caveats("pub fn f() -> u32 { 1 }");
        assert_eq!(caveats[0].status, "unknown-source-scanner-only");
    }
}
