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
const NORMALIZATION_ALGORITHM: &str = "octet-source-token-v1";
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
