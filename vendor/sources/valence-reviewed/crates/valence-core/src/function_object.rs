use std::error::Error;
use std::fmt;

use serde::Serialize;

pub const FUNCTION_OBJECT_SCHEMA: &str = "octet.function-object.v1";
pub const NORMALIZATION_ALGORITHM: &str = "octet-source-token-v1";
/// Label for Octet's literal-sensitive source tokens; the scanner lives outside this crate.
pub const NORMALIZATION_ALGORITHM_V2: &str = "octet-source-token-v2";
const HASH_ALGORITHM: &str = "BLAKE3";

/// Typed, algorithm-tagged normalized function source for Function Object receipts.
///
/// Carries the normalization algorithm identifier alongside the normalized bytes
/// so the selected tokenization version becomes part of the object identity.
/// Construction rejects unknown labels; it does not verify that the caller's
/// normalized bytes match its raw source or the declared algorithm.
///
/// `valence-core` owns the versioned algorithm labels and typed contract;
/// `cargo-octet` owns the source-token scanner that produces the bytes.
#[derive(Debug, Clone)]
pub struct NormalizedFunctionSource {
    algorithm: &'static str,
    normalized: String,
}

impl NormalizedFunctionSource {
    /// Create a typed normalized source with a supported algorithm label.
    ///
    /// # Errors
    ///
    /// Returns [`ConstructionError::NormalizationAlgorithm`] if `algorithm`
    /// is neither [`NORMALIZATION_ALGORITHM`] nor [`NORMALIZATION_ALGORITHM_V2`].
    pub fn new(algorithm: &'static str, normalized: String) -> Result<Self, ConstructionError> {
        if algorithm != NORMALIZATION_ALGORITHM && algorithm != NORMALIZATION_ALGORITHM_V2 {
            return Err(ConstructionError::NormalizationAlgorithm {
                algorithm: algorithm.to_owned(),
            });
        }
        Ok(Self { algorithm, normalized })
    }

    /// The normalization algorithm declared by the caller for these bytes.
    #[must_use]
    pub fn algorithm(&self) -> &'static str {
        self.algorithm
    }

    /// The normalized function source bytes.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.normalized
    }
}

/// Typed construction error returned by Function Object v1 core construction.
///
/// The serialized Function Object v1 DTO never carries these errors: typed
/// variants live only at the Rust boundary and in internal construction logic,
/// while the stable strings emitted by [`DependencyIdentityStatus`], effect/macro
/// caveat classifications, and pure-cache blockers remain byte-identical to the
/// existing v1 contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionError {
    /// The caller tagged normalized source with an unsupported algorithm.
    NormalizationAlgorithm { algorithm: String },
    /// The canonical BLAKE3 hash payload could not be serialized.
    CanonicalSerialization { detail: String },
    /// The caller provided a dependency identity status string that is not one
    /// of the supported Function Object v1 dependency identity statuses.
    UnknownDependencyIdentityStatus { status: String },
}

impl fmt::Display for ConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NormalizationAlgorithm { algorithm } => write!(
                f,
                "unsupported function object normalization algorithm `{algorithm}`; expected `{NORMALIZATION_ALGORITHM}` or `{NORMALIZATION_ALGORITHM_V2}`"
            ),
            Self::CanonicalSerialization { detail } => write!(f, "canonicalize function object: {detail}"),
            Self::UnknownDependencyIdentityStatus { status } => {
                write!(f, "unknown function object dependency identity status `{status}`")
            }
        }
    }
}

impl Error for ConstructionError {}

/// Typed Function Object v1 dependency identity status.
///
/// Serializes (via [`DependencyIdentityStatus::as_str`]) to the existing stable
/// v1 dependency status strings used in caveat records and corpus counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyIdentityStatus {
    SourceScannerTokenSet,
    UnsupportedMacroContext,
    Resolved,
}

impl DependencyIdentityStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceScannerTokenSet => "source-scanner-token-set",
            Self::UnsupportedMacroContext => "unsupported-macro-context",
            Self::Resolved => "resolved",
        }
    }
}

impl std::str::FromStr for DependencyIdentityStatus {
    type Err = ConstructionError;

    /// Parse a stable v1 dependency identity status string into the typed
    /// variant, failing closed for unknown values.
    ///
    /// # Errors
    ///
    /// Returns [`ConstructionError::UnknownDependencyIdentityStatus`] if the
    /// input is not one of the supported v1 dependency identity statuses.
    fn from_str(status: &str) -> Result<Self, Self::Err> {
        match status {
            "source-scanner-token-set" => Ok(Self::SourceScannerTokenSet),
            "unsupported-macro-context" => Ok(Self::UnsupportedMacroContext),
            "resolved" => Ok(Self::Resolved),
            other => Err(ConstructionError::UnknownDependencyIdentityStatus {
                status: other.to_owned(),
            }),
        }
    }
}

/// Typed Function Object v1 effect caveat kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCaveatKind {
    Unsafe,
    Panic,
    Filesystem,
    Network,
    Process,
    Time,
    EffectSummary,
}

impl EffectCaveatKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unsafe => "unsafe",
            Self::Panic => "panic",
            Self::Filesystem => "filesystem",
            Self::Network => "network",
            Self::Process => "process",
            Self::Time => "time",
            Self::EffectSummary => "effect_summary",
        }
    }
}

/// Typed Function Object v1 effect caveat status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCaveatStatus {
    SourceTokenCaveat,
    UnknownSourceScannerOnly,
}

impl EffectCaveatStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceTokenCaveat => "source-token-caveat",
            Self::UnknownSourceScannerOnly => "unknown-source-scanner-only",
        }
    }
}

/// Typed Function Object v1 macro caveat status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroCaveatStatus {
    ReviewRequired,
    NoneDetected,
}

impl MacroCaveatStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReviewRequired => "review-required",
            Self::NoneDetected => "none-detected",
        }
    }
}

/// Typed Function Object v1 pure-cache blocker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PureCacheBlocker {
    DependencyIdentityUnresolved,
    EffectSummaryUnresolved,
    MacroExpansionUnmodeled,
}

impl PureCacheBlocker {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DependencyIdentityUnresolved => "dependency-identity-unresolved",
            Self::EffectSummaryUnresolved => "effect-summary-unresolved",
            Self::MacroExpansionUnmodeled => "macro-expansion-unmodeled",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionObjectInput<'a> {
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub name: &'a str,
    pub source_path: String,
    pub source_span: SourceSpan,
    pub function_source: &'a str,
    pub normalized_function_source: NormalizedFunctionSource,
    pub dependency_set_hash: String,
    pub dependency_identity_status: DependencyIdentityStatus,
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

/// Reviewable typed hash material for Function Object v1 object identity.
///
/// Exactly these fields are included in the canonical BLAKE3 object hash.
/// Display and source-local fields (`source_path`, `source_span`) and the
/// computed `object_hash` itself are deliberately excluded so the identity is
/// stable across path relocation and independent of its own hash output.
///
/// Field declaration order matches the historical inline `serde_json::json!`
/// insertion order. Canonical JSON is routed through [`serde_json::to_value`]
/// before [`serde_json::to_string`] so that, with the default `serde_json`
/// backend (no `preserve_order` feature), object keys serialize in the same
/// sorted order the previous inline payload produced. This keeps existing
/// object hashes byte-stable; do not switch to direct `serde_json::to_string`
/// serialization of this struct without refreshing every object corpus fixture
/// and recording an explicit migration.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionObjectHashPayload<'a> {
    pub schema: &'a str,
    pub semantic_id: &'a str,
    pub name: &'a str,
    pub body_hash: &'a str,
    pub normalized_body_hash: &'a str,
    pub signature_hash: &'a str,
    pub dependency_set_hash: &'a str,
    pub dependency_caveats: &'a [DependencyCaveat],
    pub effect_caveats: &'a [EffectCaveat],
    pub macro_caveats: &'a [MacroCaveat],
    pub normalization: &'a NormalizationMetadata,
    pub caveat_summary: &'a CaveatSummary,
    pub pure_cache_eligibility: &'a PureCacheEligibility,
    pub correctness_claim: &'a CorrectnessClaim,
}

impl<'a> FunctionObjectHashPayload<'a> {
    /// Build the reviewable hash payload for a constructed Function Object v1.
    #[must_use]
    pub fn from_object(object: &'a FunctionObjectV1) -> Self {
        Self {
            schema: object.schema,
            semantic_id: &object.semantic_id,
            name: &object.name,
            body_hash: &object.body_hash,
            normalized_body_hash: &object.normalized_body_hash,
            signature_hash: &object.signature_hash,
            dependency_set_hash: &object.dependency_set_hash,
            dependency_caveats: &object.dependency_caveats,
            effect_caveats: &object.effect_caveats,
            macro_caveats: &object.macro_caveats,
            normalization: &object.normalization,
            caveat_summary: &object.caveat_summary,
            pure_cache_eligibility: &object.pure_cache_eligibility,
            correctness_claim: &object.correctness_claim,
        }
    }

    /// Canonical JSON for this payload, with keys serialized in the sorted
    /// order produced by the default `serde_json` backend so existing object
    /// hashes remain byte-stable.
    ///
    /// # Errors
    ///
    /// Returns [`ConstructionError::CanonicalSerialization`] if the payload
    /// cannot be serialized into canonical JSON.
    pub fn canonical_json(&self) -> Result<String, ConstructionError> {
        let value = serde_json::to_value(self).map_err(|error| ConstructionError::CanonicalSerialization {
            detail: error.to_string(),
        })?;
        serde_json::to_string(&value).map_err(|error| ConstructionError::CanonicalSerialization {
            detail: error.to_string(),
        })
    }

    /// Canonical BLAKE3 identity for this payload.
    ///
    /// # Errors
    ///
    /// Returns [`ConstructionError::CanonicalSerialization`] if the payload
    /// cannot be serialized into canonical JSON.
    pub fn canonical_hash(&self) -> Result<String, ConstructionError> {
        let bytes = self.canonical_json()?;
        Ok(full_hash(&bytes))
    }
}

/// Builds a deterministic Function Object v1 receipt from typed source inputs.
///
/// # Errors
///
/// Returns [`ConstructionError`] if the resulting object cannot be serialized
/// into the canonical JSON payload used for BLAKE3 object identity.
pub fn build_function_object(input: FunctionObjectInput<'_>) -> Result<FunctionObjectV1, ConstructionError> {
    let algorithm = input.normalized_function_source.algorithm();
    let normalized = input.normalized_function_source.as_str();
    let normalized_body_hash = full_hash(normalized);
    let body_hash = normalized_body_hash.clone();
    let signature_hash = signature_hash(normalized);
    let dependency_caveats = dependency_caveats(input.dependency_identity_status);
    let effect_caveats = effect_caveats_from_normalized(normalized, algorithm);
    let macro_caveats = macro_caveats(input.macro_calls, algorithm);
    let caveat_summary = caveat_summary(&dependency_caveats, &effect_caveats, &macro_caveats);
    let pure_cache_eligibility = pure_cache_eligibility(&dependency_caveats, &effect_caveats, &macro_caveats);
    let semantic_id = short_hash(&format!(
        "schema={FUNCTION_OBJECT_SCHEMA}\nname={}\nbody={normalized_body_hash}\nsignature={signature_hash}\ndeps={}\ndep_status={}\nnormalization={algorithm}\ntool={}\n",
        input.name,
        input.dependency_set_hash,
        input.dependency_identity_status.as_str(),
        input.tool_version,
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
            algorithm,
            tool_name: input.tool_name,
            tool_version: input.tool_version,
            hash_algorithm: HASH_ALGORITHM,
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

/// Computes the canonical BLAKE3 identity for a Function Object v1 receipt.
///
/// # Errors
///
/// Returns [`ConstructionError::CanonicalSerialization`] if the object cannot
/// be serialized into the canonical JSON payload used for hashing.
pub fn object_hash(object: &FunctionObjectV1) -> Result<String, ConstructionError> {
    FunctionObjectHashPayload::from_object(object).canonical_hash()
}

/// Returns the typed effect caveats scanned from raw function source using the
/// coarse whitespace-collapse tokenization.
///
/// [`build_function_object`] scans supplied normalized source
/// (`NormalizedFunctionSource`); this public helper uses a distinct coarse
/// whitespace tokenization and retains the historical v1 caveat wording
/// for compatibility with existing callers that scan raw source directly.
#[must_use]
pub fn effect_caveats(function_source: &str) -> Vec<EffectCaveat> {
    let normalized = coarse_whitespace_tokens(function_source);
    effect_caveats_from_normalized(&normalized, NORMALIZATION_ALGORITHM)
}

fn effect_caveats_from_normalized(normalized: &str, algorithm: &str) -> Vec<EffectCaveat> {
    let mut caveats = Vec::new();
    for (needle, kind, detail) in [
        ("unsafe", EffectCaveatKind::Unsafe, "Source tokens contain unsafe; effect summary remains caveated."),
        (
            "panic",
            EffectCaveatKind::Panic,
            "Source tokens contain panic-like spelling; effect summary remains caveated.",
        ),
        (
            "unwrap",
            EffectCaveatKind::Panic,
            "Source tokens contain unwrap-like spelling; effect summary remains caveated.",
        ),
        (
            "expect",
            EffectCaveatKind::Panic,
            "Source tokens contain expect-like spelling; effect summary remains caveated.",
        ),
        (
            "fs",
            EffectCaveatKind::Filesystem,
            "Source tokens contain filesystem-like spelling; effect summary remains caveated.",
        ),
        (
            "net",
            EffectCaveatKind::Network,
            "Source tokens contain network-like spelling; effect summary remains caveated.",
        ),
        (
            "Command",
            EffectCaveatKind::Process,
            "Source tokens contain process-spawn-like spelling; effect summary remains caveated.",
        ),
        (
            "SystemTime",
            EffectCaveatKind::Time,
            "Source tokens contain wall-clock-like spelling; effect summary remains caveated.",
        ),
    ] {
        if normalized.split_whitespace().any(|token| token == needle) {
            caveats.push(EffectCaveat {
                kind: kind.as_str(),
                status: EffectCaveatStatus::SourceTokenCaveat.as_str(),
                detail,
            });
        }
    }
    if caveats.is_empty() {
        caveats.push(EffectCaveat {
            kind: EffectCaveatKind::EffectSummary.as_str(),
            status: EffectCaveatStatus::UnknownSourceScannerOnly.as_str(),
            detail: if algorithm == NORMALIZATION_ALGORITHM {
                "No coarse ambient effect token was found, but v1 source scanning does not prove purity."
            } else {
                "No coarse ambient effect token was found in supplied normalized source; source scanning does not prove purity."
            },
        });
    }
    caveats
}

fn signature_hash(normalized: &str) -> String {
    let signature = normalized.split('{').next().unwrap_or(normalized);
    full_hash(&coarse_whitespace_tokens(signature))
}

fn dependency_caveats(status: DependencyIdentityStatus) -> Vec<DependencyCaveat> {
    let detail = match status {
        DependencyIdentityStatus::UnsupportedMacroContext => {
            "Dependency identity is token-derived and the function contains macro calls; dependencies are not resolved through rustc/HIR."
        }
        _ => "Dependency identity is a source-scanner token set, not a resolved dependency graph.",
    };
    vec![DependencyCaveat {
        kind: "dependency_identity",
        status: status.as_str(),
        detail,
    }]
}

fn macro_caveats(macro_calls: bool, algorithm: &str) -> Vec<MacroCaveat> {
    let (status, detail) = if macro_calls {
        (
            MacroCaveatStatus::ReviewRequired,
            if algorithm == NORMALIZATION_ALGORITHM {
                "Function body contains a macro call; v1 object identity is based on source tokens, not expanded macro output."
            } else {
                "Caller reports a macro call; object identity is based on supplied source tokens, not expanded macro output."
            },
        )
    } else {
        (
            MacroCaveatStatus::NoneDetected,
            if algorithm == NORMALIZATION_ALGORITHM {
                "No macro call was detected by the v1 source scanner."
            } else {
                "No macro call was reported in caller-supplied source facts."
            },
        )
    };
    vec![MacroCaveat {
        kind: "macro_call",
        status: status.as_str(),
        detail,
    }]
}

fn caveat_summary(
    dependency_caveats: &[DependencyCaveat],
    effect_caveats: &[EffectCaveat],
    macro_caveats: &[MacroCaveat],
) -> CaveatSummary {
    let dependency_identity =
        if dependency_caveats.iter().any(|caveat| caveat.status == DependencyIdentityStatus::Resolved.as_str()) {
            "resolved"
        } else {
            "unresolved-source-scanner"
        };
    let macro_expansion =
        if macro_caveats.iter().any(|caveat| caveat.status == MacroCaveatStatus::ReviewRequired.as_str()) {
            "macro-expansion-not-modeled"
        } else {
            "none-detected"
        };
    let effect_summary =
        if effect_caveats.iter().any(|caveat| caveat.status == EffectCaveatStatus::SourceTokenCaveat.as_str()) {
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
    if dependency_caveats.iter().any(|caveat| caveat.status != DependencyIdentityStatus::Resolved.as_str()) {
        blockers.push(PureCacheBlocker::DependencyIdentityUnresolved.as_str());
    }
    if effect_caveats.iter().any(|caveat| caveat.status != "resolved-pure") {
        blockers.push(PureCacheBlocker::EffectSummaryUnresolved.as_str());
    }
    if macro_caveats.iter().any(|caveat| caveat.status == MacroCaveatStatus::ReviewRequired.as_str()) {
        blockers.push(PureCacheBlocker::MacroExpansionUnmodeled.as_str());
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

/// Coarse whitespace-collapse tokenization used for signature hashing and raw
/// effect scanning.
///
/// This is distinct from the Function Object v1 normalization algorithm
/// (`octet-source-token-v1`) that produces `NormalizedFunctionSource`: the v1
/// algorithm strips comments/strings and tokenizes by identifier characters in
/// `cargo-octet`, while this helper only collapses adjacent whitespace.
fn coarse_whitespace_tokens(source: &str) -> String {
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
    use std::str::FromStr;

    use super::*;

    fn input_with_normalization<'a>(
        function_source: &'a str,
        normalized: String,
        algorithm: &'static str,
    ) -> FunctionObjectInput<'a> {
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
            normalized_function_source: NormalizedFunctionSource::new(algorithm, normalized)
                .expect("supported normalization label"),
            dependency_set_hash: full_hash("answer"),
            dependency_identity_status: DependencyIdentityStatus::SourceScannerTokenSet,
            macro_calls: false,
        }
    }

    fn input(function_source: &str) -> FunctionObjectInput<'_> {
        input_with_normalization(function_source, coarse_whitespace_tokens(function_source), NORMALIZATION_ALGORITHM)
    }

    fn input_with_macro(function_source: &str) -> FunctionObjectInput<'_> {
        FunctionObjectInput {
            macro_calls: true,
            dependency_identity_status: DependencyIdentityStatus::UnsupportedMacroContext,
            ..input(function_source)
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
    fn pinned_v1_canonical_payload_and_object_identity_are_unchanged() {
        // Captured from the public constructor at standalone Valence
        // 5f1c2ba5072c6f9622fa59b1af20502985f569fd, before v2 admission.
        let object = build_function_object(input("pub fn answer() -> u32 { 42 }")).unwrap();
        let payload = FunctionObjectHashPayload::from_object(&object).canonical_json().unwrap();
        let golden = r#"{"body_hash":"b3:ced95733616d5c172c9a4a368f3c496ab22405e9ca2d0db5562d4cb5a74c63fb","caveat_summary":{"cache_policy":"pure-cache-blocked-unless-external-policy-accepts-caveats","dependency_identity":"unresolved-source-scanner","effect_summary":"unknown-not-pure-proof","macro_expansion":"none-detected"},"correctness_claim":{"claims_correctness":false,"detail":"Function object hashes identify reviewed source evidence and caveats; they do not prove behavioral correctness."},"dependency_caveats":[{"detail":"Dependency identity is a source-scanner token set, not a resolved dependency graph.","kind":"dependency_identity","status":"source-scanner-token-set"}],"dependency_set_hash":"b3:63b24a318813c47bce54f4e4dc1f052a586c63eeb96d20ed88bf1b86bd6d362c","effect_caveats":[{"detail":"No coarse ambient effect token was found, but v1 source scanning does not prove purity.","kind":"effect_summary","status":"unknown-source-scanner-only"}],"macro_caveats":[{"detail":"No macro call was detected by the v1 source scanner.","kind":"macro_call","status":"none-detected"}],"name":"answer","normalization":{"algorithm":"octet-source-token-v1","hash_algorithm":"BLAKE3","tool_name":"cargo-octet","tool_version":"0.1.0"},"normalized_body_hash":"b3:ced95733616d5c172c9a4a368f3c496ab22405e9ca2d0db5562d4cb5a74c63fb","pure_cache_eligibility":{"blockers":["dependency-identity-unresolved","effect-summary-unresolved"],"eligible":false,"status":"blocked-by-caveats"},"schema":"octet.function-object.v1","semantic_id":"b3:560b33d500ca1383","signature_hash":"b3:cf479cc5354ccd63dedce8c7f1397a25850c0ba452a99d0dc4423e14cb9328e6"}"#;
        assert_eq!(payload, golden);
        assert_eq!(object.semantic_id, "b3:560b33d500ca1383");
        assert_eq!(object.object_hash, "b3:da550fa4897c0b6b3ba9adb819eb1c1bedf5f5b6ad6c1c4498d9afef0719c9b4");
    }

    #[test]
    fn function_object_records_effect_caveats_without_claiming_purity() {
        let object = build_function_object(input("pub fn answer() -> u32 { let unwrap = 1; unwrap }")).unwrap();
        assert!(object.effect_caveats.iter().any(|caveat| caveat.kind == "panic"));
        assert!(!object.pure_cache_eligibility.eligible);
        assert!(object.pure_cache_eligibility.blockers.contains(&"effect-summary-unresolved"));
    }

    #[test]
    fn core_owned_normalization_is_stable_across_constructions() {
        // r[octet.valence_project_extraction.phase4.function_object_normalization.core_owned]
        let source = "pub fn checked(input: u32) -> u32 { input + 1 }\n";
        let facts = NormalizedFunctionSource::new(NORMALIZATION_ALGORITHM, coarse_whitespace_tokens(source))
            .expect("v1 normalized source");
        let first = build_function_object(FunctionObjectInput {
            tool_name: "cargo-octet",
            tool_version: "0.1.0",
            name: "checked",
            source_path: "src/lib.rs".to_owned(),
            source_span: SourceSpan {
                start_line: 1,
                end_line: 1,
            },
            function_source: source,
            normalized_function_source: facts.clone(),
            dependency_set_hash: full_hash("checked"),
            dependency_identity_status: DependencyIdentityStatus::SourceScannerTokenSet,
            macro_calls: false,
        })
        .unwrap();
        let second = build_function_object(FunctionObjectInput {
            tool_name: "cargo-octet",
            tool_version: "0.1.0",
            name: "checked",
            source_path: "src/lib.rs".to_owned(),
            source_span: SourceSpan {
                start_line: 1,
                end_line: 1,
            },
            function_source: source,
            normalized_function_source: facts,
            dependency_set_hash: full_hash("checked"),
            dependency_identity_status: DependencyIdentityStatus::SourceScannerTokenSet,
            macro_calls: false,
        })
        .unwrap();
        assert_eq!(first.normalized_body_hash, second.normalized_body_hash);
        assert_eq!(first.semantic_id, second.semantic_id);
        assert_eq!(first.object_hash, second.object_hash);
        assert!(!first.correctness_claim.claims_correctness);
    }

    #[test]
    fn normalization_mismatch_fails_closed_without_object_hash() {
        // r[octet.valence_project_extraction.phase4.function_object_normalization.rejects_mismatch]
        // The typed contract fails closed at the core boundary before any object
        // hash can be produced: a mismatched algorithm cannot reach
        // build_function_object.
        let unsupported = NormalizedFunctionSource::new("other-algorithm", "pub fn answer ( ) { 42 }".to_owned());
        assert!(matches!(
            unsupported,
            Err(ConstructionError::NormalizationAlgorithm { algorithm }) if algorithm == "other-algorithm"
        ));
        let mismatched = NormalizedFunctionSource::new("octet-source-token-v0", String::new());
        assert!(matches!(mismatched, Err(ConstructionError::NormalizationAlgorithm { .. })));
        for algorithm in [NORMALIZATION_ALGORITHM, NORMALIZATION_ALGORITHM_V2] {
            let valid = NormalizedFunctionSource::new(algorithm, "pub fn answer ( ) { 42 }".to_owned())
                .expect("supported normalization label");
            assert_eq!(valid.algorithm(), algorithm);
        }
    }

    #[test]
    fn normalization_version_separates_semantic_and_object_identity() {
        let tokens = "pub fn answer ( ) -> u32 { 42 }";
        let v1 = build_function_object(input_with_normalization(
            "pub fn answer() -> u32 { 42 }",
            tokens.to_owned(),
            NORMALIZATION_ALGORITHM,
        ))
        .unwrap();
        let v2 = build_function_object(input_with_normalization(
            "pub fn answer() -> u32 { 42 }",
            tokens.to_owned(),
            NORMALIZATION_ALGORITHM_V2,
        ))
        .unwrap();

        assert_eq!(v1.schema, v2.schema);
        assert_eq!(v1.normalized_body_hash, v2.normalized_body_hash);
        assert_eq!(v1.signature_hash, v2.signature_hash);
        assert_eq!(v1.normalization.algorithm, NORMALIZATION_ALGORITHM);
        assert_eq!(v2.normalization.algorithm, NORMALIZATION_ALGORITHM_V2);
        assert_ne!(v1.semantic_id, v2.semantic_id);
        assert_ne!(v1.object_hash, v2.object_hash);
        assert_eq!(FunctionObjectHashPayload::from_object(&v2).canonical_hash().unwrap(), v2.object_hash);
        assert_eq!(v1.correctness_claim.claims_correctness, v2.correctness_claim.claims_correctness);
        assert!(!v2.correctness_claim.claims_correctness);
        assert!(!v1.pure_cache_eligibility.eligible);
        assert_eq!(v1.pure_cache_eligibility.blockers, v2.pure_cache_eligibility.blockers);
        assert_eq!(v1.effect_caveats[0].status, v2.effect_caveats[0].status);
        assert_eq!(v1.macro_caveats[0].status, v2.macro_caveats[0].status);
        assert!(!v2.effect_caveats[0].detail.contains("v1 source scanning"));
        assert!(!v2.macro_caveats[0].detail.contains("v1 source scanner"));
    }

    #[test]
    fn v2_caller_supplied_literal_and_quoted_url_tokens_affect_identity() {
        // The core consumes supplied tokens; it does not lex source or verify the
        // caller's tag. The Octet scanner must supply literal-sensitive v2 bytes.
        let source = "pub fn answer() -> u32 { let _url = \"https://example.invalid/a\"; 1 }";
        let original_tokens = "pub fn answer ( ) -> u32 { let _url = \"https://example.invalid/a\" ; 1 }";
        let changed_literal = "pub fn answer ( ) -> u32 { let _url = \"https://example.invalid/b\" ; 1 }";
        let changed_tail = "pub fn answer ( ) -> u32 { let _url = \"https://example.invalid/a\" ; 2 }";
        let original = build_function_object(input_with_normalization(
            source,
            original_tokens.to_owned(),
            NORMALIZATION_ALGORITHM_V2,
        ))
        .unwrap();
        for tokens in [changed_literal, changed_tail] {
            let changed =
                build_function_object(input_with_normalization(source, tokens.to_owned(), NORMALIZATION_ALGORITHM_V2))
                    .unwrap();
            assert_ne!(original.normalized_body_hash, changed.normalized_body_hash);
            assert_ne!(original.semantic_id, changed.semantic_id);
            assert_ne!(original.object_hash, changed.object_hash);
            assert!(!changed.correctness_claim.claims_correctness);
            assert!(!changed.pure_cache_eligibility.eligible);
            assert!(changed.pure_cache_eligibility.blockers.contains(&"effect-summary-unresolved"));
        }
    }

    #[test]
    fn v2_reported_macro_context_stays_unresolved_and_source_bound() {
        let object = build_function_object(FunctionObjectInput {
            macro_calls: true,
            dependency_identity_status: DependencyIdentityStatus::UnsupportedMacroContext,
            ..input_with_normalization(
                "pub fn answer() { work!(); }",
                "pub fn answer ( ) { work ! ( ) ; }".to_owned(),
                NORMALIZATION_ALGORITHM_V2,
            )
        })
        .unwrap();
        assert_eq!(object.macro_caveats[0].status, MacroCaveatStatus::ReviewRequired.as_str());
        assert!(object.macro_caveats[0].detail.contains("supplied source tokens"));
        assert!(!object.macro_caveats[0].detail.contains("v1 object"));
        assert!(!object.pure_cache_eligibility.eligible);
        assert!(object.pure_cache_eligibility.blockers.contains(&"macro-expansion-unmodeled"));
        assert!(!object.correctness_claim.claims_correctness);
    }

    #[test]
    fn hash_payload_is_reviewable_and_excludes_display_fields() {
        // r[octet.valence_project_extraction.phase4.function_object_hash_payload.reviewable]
        let object = build_function_object(input("pub fn answer() -> u32 { 42 }")).unwrap();
        let payload = FunctionObjectHashPayload::from_object(&object);
        let json = payload.canonical_json().unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let map = value.as_object().expect("hash payload object");
        let mut keys: Vec<&str> = map.keys().map(|key| key.as_str()).collect();
        keys.sort();
        let mut expected = vec![
            "body_hash",
            "caveat_summary",
            "correctness_claim",
            "dependency_caveats",
            "dependency_set_hash",
            "effect_caveats",
            "macro_caveats",
            "name",
            "normalized_body_hash",
            "normalization",
            "pure_cache_eligibility",
            "schema",
            "semantic_id",
            "signature_hash",
        ];
        expected.sort();
        assert_eq!(keys, expected, "hash payload must include exactly the reviewed hash-material fields");
        for excluded in ["source_path", "source_span", "object_hash"] {
            assert!(map.get(excluded).is_none(), "hash payload must exclude `{excluded}`: {value:#?}");
        }
        // The reviewable payload must reproduce the object's canonical hash.
        assert_eq!(payload.canonical_hash().unwrap(), object.object_hash);
        // Existing object corpus hashes remain stable because canonical JSON
        // keys serialize in the sorted order produced by the default serde_json
        // backend (no `preserve_order` feature).
        assert_eq!(
            map.keys().next().unwrap(),
            "body_hash",
            "canonical JSON keys must be sorted by the default serde_json backend"
        );
    }

    #[test]
    fn legacy_inline_json_payload_matches_new_canonical_json() {
        let object = build_function_object(input("pub fn answer() -> u32 { 42 }")).unwrap();
        let legacy_payload = serde_json::json!({
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
        let legacy_bytes = serde_json::to_string(&legacy_payload).unwrap();
        let new_bytes = FunctionObjectHashPayload::from_object(&object).canonical_json().unwrap();
        assert_eq!(
            legacy_bytes, new_bytes,
            "new canonical hash payload must be byte-identical to the legacy inline json payload"
        );
    }

    #[test]
    fn typed_contracts_serialize_to_existing_v1_strings() {
        // r[octet.valence_project_extraction.phase4.function_object_typed_api.serializes_compatibly]
        assert_eq!(DependencyIdentityStatus::SourceScannerTokenSet.as_str(), "source-scanner-token-set");
        assert_eq!(DependencyIdentityStatus::UnsupportedMacroContext.as_str(), "unsupported-macro-context");
        assert_eq!(DependencyIdentityStatus::Resolved.as_str(), "resolved");
        assert_eq!(EffectCaveatKind::Unsafe.as_str(), "unsafe");
        assert_eq!(EffectCaveatKind::EffectSummary.as_str(), "effect_summary");
        assert_eq!(EffectCaveatStatus::SourceTokenCaveat.as_str(), "source-token-caveat");
        assert_eq!(EffectCaveatStatus::UnknownSourceScannerOnly.as_str(), "unknown-source-scanner-only");
        assert_eq!(MacroCaveatStatus::ReviewRequired.as_str(), "review-required");
        assert_eq!(MacroCaveatStatus::NoneDetected.as_str(), "none-detected");
        assert_eq!(PureCacheBlocker::DependencyIdentityUnresolved.as_str(), "dependency-identity-unresolved");
        assert_eq!(PureCacheBlocker::EffectSummaryUnresolved.as_str(), "effect-summary-unresolved");
        assert_eq!(PureCacheBlocker::MacroExpansionUnmodeled.as_str(), "macro-expansion-unmodeled");
        for status in [
            DependencyIdentityStatus::SourceScannerTokenSet,
            DependencyIdentityStatus::UnsupportedMacroContext,
            DependencyIdentityStatus::Resolved,
        ] {
            assert_eq!(
                DependencyIdentityStatus::from_str(status.as_str()).unwrap(),
                status,
                "dependency identity status must round-trip through its stable string"
            );
        }
    }

    #[test]
    fn malformed_dependency_identity_status_fails_with_typed_error() {
        let error = DependencyIdentityStatus::from_str("not-a-real-status").unwrap_err();
        assert_eq!(error, ConstructionError::UnknownDependencyIdentityStatus {
            status: "not-a-real-status".to_owned()
        });
    }

    #[test]
    fn macro_context_dependency_status_serializes_to_stable_string() {
        let object = build_function_object(input_with_macro("pub fn checked() -> u32 { vec![1]; 0 }")).unwrap();
        assert_eq!(object.dependency_caveats[0].status, DependencyIdentityStatus::UnsupportedMacroContext.as_str());
        assert_eq!(object.macro_caveats[0].status, MacroCaveatStatus::ReviewRequired.as_str());
        assert!(object.pure_cache_eligibility.blockers.contains(&"macro-expansion-unmodeled"));
    }
}
