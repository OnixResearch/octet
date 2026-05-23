use serde::Serialize;

pub const FUNCTION_OBJECT_SCHEMA: &str = "octet.function-object.v1";
pub const NORMALIZATION_ALGORITHM: &str = "octet-source-token-v1";
const DEPENDENCY_STATUS_MACRO_CONTEXT: &str = "unsupported-macro-context";

#[derive(Debug, Clone)]
pub struct FunctionObjectInput<'a> {
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub name: &'a str,
    pub source_path: String,
    pub source_span: SourceSpan,
    pub function_source: &'a str,
    pub normalized_function_source: String,
    pub dependency_set_hash: String,
    pub dependency_identity_status: &'static str,
    pub macro_calls: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionObjectV1 {
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
pub struct SourceSpan {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyCaveat {
    pub kind: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectCaveat {
    pub kind: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroCaveat {
    pub kind: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizationMetadata {
    pub algorithm: &'static str,
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub hash_algorithm: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrectnessClaim {
    pub claims_correctness: bool,
    pub detail: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaveatSummary {
    pub dependency_identity: &'static str,
    pub macro_expansion: &'static str,
    pub effect_summary: &'static str,
    pub cache_policy: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PureCacheEligibility {
    pub eligible: bool,
    pub status: &'static str,
    pub blockers: Vec<&'static str>,
}

pub fn build_function_object(input: FunctionObjectInput<'_>) -> Result<FunctionObjectV1, String> {
    let normalized_body_hash = full_hash(&input.normalized_function_source);
    let body_hash = normalized_body_hash.clone();
    let signature_hash = signature_hash(&input.normalized_function_source);
    let dependency_caveats = dependency_caveats(input.dependency_identity_status);
    let effect_caveats = effect_caveats_from_normalized(&input.normalized_function_source);
    let macro_caveats = macro_caveats(input.macro_calls);
    let caveat_summary = caveat_summary(&dependency_caveats, &effect_caveats, &macro_caveats);
    let pure_cache_eligibility = pure_cache_eligibility(&dependency_caveats, &effect_caveats, &macro_caveats);
    let semantic_id = short_hash(&format!(
        "schema={FUNCTION_OBJECT_SCHEMA}\nname={}\nbody={normalized_body_hash}\nsignature={signature_hash}\ndeps={}\ndep_status={}\nnormalization={NORMALIZATION_ALGORITHM}\ntool={}\n",
        input.name, input.dependency_set_hash, input.dependency_identity_status, input.tool_version,
    ));
    let mut object = FunctionObjectV1 {
        schema: FUNCTION_OBJECT_SCHEMA,
        object_hash: String::new(),
        semantic_id,
        name: input.name.to_owned(),
        source_path: input.source_path,
        source_span: input.source_span,
        body_hash,
        normalized_body_hash,
        signature_hash,
        dependency_set_hash: input.dependency_set_hash,
        dependency_caveats,
        effect_caveats,
        macro_caveats,
        normalization: NormalizationMetadata {
            algorithm: NORMALIZATION_ALGORITHM,
            tool_name: input.tool_name,
            tool_version: input.tool_version,
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

pub fn object_hash(object: &FunctionObjectV1) -> Result<String, String> {
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
    Ok(full_hash(&bytes))
}

pub fn effect_caveats(function_source: &str) -> Vec<EffectCaveat> {
    let normalized = normalize_tokens(function_source);
    effect_caveats_from_normalized(&normalized)
}

fn effect_caveats_from_normalized(normalized: &str) -> Vec<EffectCaveat> {
    let mut caveats = Vec::new();
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

fn signature_hash(function_source: &str) -> String {
    let signature = function_source.split('{').next().unwrap_or(function_source);
    full_hash(&normalize_tokens(signature))
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

fn normalize_tokens(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn short_hash(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes()).to_hex().to_string();
    format!("b3:{}", &hash[..16])
}

fn full_hash(input: &str) -> String {
    format!("b3:{}", blake3::hash(input.as_bytes()).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input<'a>(function_source: &'a str) -> FunctionObjectInput<'a> {
        FunctionObjectInput {
            tool_name: "cargo-octet",
            tool_version: "0.1.0",
            name: "answer",
            source_path: "src/lib.rs".to_owned(),
            source_span: SourceSpan {
                start_line: 1,
                end_line: 1,
            },
            function_source,
            normalized_function_source: normalize_tokens(function_source),
            dependency_set_hash: full_hash("answer"),
            dependency_identity_status: "source-token-set",
            macro_calls: false,
        }
    }

    #[test]
    fn function_object_hash_is_stable_for_same_core_inputs() {
        let one = build_function_object(input("pub fn answer() -> u32 { 42 }")).unwrap();
        let two = build_function_object(input("pub fn answer() -> u32 { 42 }")).unwrap();
        assert_eq!(one.object_hash, two.object_hash);
        assert_eq!(one.schema, FUNCTION_OBJECT_SCHEMA);
        assert!(!one.correctness_claim.claims_correctness);
    }

    #[test]
    fn function_object_records_effect_caveats_without_claiming_purity() {
        let object = build_function_object(input("pub fn answer() -> u32 { let unwrap = 1; unwrap }")).unwrap();
        assert!(object.effect_caveats.iter().any(|caveat| caveat.kind == "panic"));
        assert!(!object.pure_cache_eligibility.eligible);
        assert!(object.pure_cache_eligibility.blockers.contains(&"effect-summary-unresolved"));
    }
}
