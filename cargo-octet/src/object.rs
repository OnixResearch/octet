use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::ObjectArgs;
use crate::cli::ObjectMode;
use crate::cli::OutputFormat;
use crate::determinism;
use crate::fingerprint;

const FUNCTION_OBJECT_SCHEMA: &str = "octet.function-object.v1";
const CORPUS_RECEIPT_SCHEMA: &str = "octet.function-object-corpus-receipt.v1";
const CORPUS_CHECK_REPORT_SCHEMA: &str = "octet.function-object-corpus-check-report.v1";
const CORPUS_RECEIPT_SCHEMA_VERSION: u32 = 1;
const NORMALIZATION_ALGORITHM: &str = "octet-source-token-v1";
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
        ObjectMode::CorpusContract => run_corpus_contract(args.output_format),
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
        boundaries: ObjectCorpusContractBoundaries {
            identity_scope: "review-and-provenance-evidence",
            claims_correctness: false,
            claims_semantic_equivalence: false,
            determinism_scope: "bounded-local-determinism",
            detail: "Contract metadata describes JSON compatibility for object corpus evidence; it does not prove behavioral correctness, semantic equivalence, cross-machine determinism, dependency resolution, macro expansion identity, or pure-cache eligibility.",
        },
    }
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

    use super::effect_caveats;
    use super::function_object_for_source;
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
    fn unknown_effects_do_not_claim_purity() {
        let caveats = effect_caveats("pub fn f() -> u32 { 1 }");
        assert_eq!(caveats[0].status, "unknown-source-scanner-only");
    }
}
