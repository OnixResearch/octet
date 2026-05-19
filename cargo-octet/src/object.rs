use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::cli::ObjectArgs;
use crate::cli::OutputFormat;
use crate::determinism;
use crate::fingerprint;

const FUNCTION_OBJECT_SCHEMA: &str = "octet.function-object.v1";
const NORMALIZATION_ALGORITHM: &str = "octet-source-token-v1";
const DEPENDENCY_STATUS_MACRO_CONTEXT: &str = "unsupported-macro-context";

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

pub fn run(args: ObjectArgs) -> Result<i32, String> {
    let mut files = Vec::new();
    for path in args.paths {
        fingerprint::collect_rust_files(&path, &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut objects = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        for span in fingerprint::find_functions(&source) {
            if let Some(selected) = args.function.as_deref()
                && selected != span.name
            {
                continue;
            }
            let function_source = &source[span.start..span.end];
            objects.push(function_object_for_source(&path, &source, &span, function_source)?);
        }
    }

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
