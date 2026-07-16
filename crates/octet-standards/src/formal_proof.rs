//! Common formal-proof envelopes and provider-specific proof contracts.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::proof::PROOF_STATUS_FAILED;
use crate::proof::PROOF_STATUS_PASSED;
use crate::proof::VerusProofReceipt;

/// Schema version for the provider-neutral formal-proof envelope.
pub const FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION: &str = "octet-formal-proof-envelope/v1";
/// Provider identifier for the Aeneas-to-Lean proof path.
pub const AENEAS_LEAN_PROVIDER_ID: &str = "aeneas-lean";
/// Provider identifier for the explicit Verus-v1 compatibility adapter.
pub const VERUS_V1_PROVIDER_ID: &str = "verus-v1-compatibility";
/// Evidence tier emitted only for a passed, fully validated formal proof.
pub const FORMAL_PROOF_EVIDENCE_TIER: &str = "formal_proof";
/// Evidence tier used for failed or recorded-only provider observations.
pub const RECORDED_ONLY_EVIDENCE_TIER: &str = "recorded_only";
/// Claim class used by the Verus compatibility adapter.
pub const VERUS_COMPATIBILITY_CLAIM_CLASS: &str = "verus-v1-compatibility";
/// Aeneas non-claim required on every common envelope.
pub const AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS: &str = "does not prove Aeneas or Charon translation soundness";
/// Aeneas non-claim required on every common envelope.
pub const AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS: &str = "does not prove Lean or verifier soundness";
/// Aeneas non-claim required on every common envelope.
pub const AENEAS_NON_CLAIM_WHOLE_PROGRAM: &str = "does not prove whole-program correctness";

const LITTLE_ENDIAN: &str = "little";
const BIG_ENDIAN: &str = "big";
const REQUIRED_AENEAS_FIELDS: &[&str] = &[
    "aeneas_release",
    "charon_revision",
    "ocaml_toolchain",
    "lean_toolchain",
    "aeneas_lean_library",
    "target_triple",
    "mir_start_stage",
    "drop_policy",
    "start_from",
    "serialization_format",
];

/// Provider-neutral status for one formal-proof execution.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalProofStatus {
    /// The provider completed successfully and passed the configured checker.
    Passed,
    /// The provider produced a failure or a non-property observation.
    Failed,
}

impl FormalProofStatus {
    /// Return the stable serialized status spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => PROOF_STATUS_PASSED,
            Self::Failed => PROOF_STATUS_FAILED,
        }
    }
}

/// Provider-neutral evidence tier for one formal-proof envelope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalProofEvidenceTier {
    /// A passed theorem receipt admitted as formal-proof evidence.
    FormalProof,
    /// A bounded observation that is not admitted as a formal proof.
    RecordedOnly,
}

impl FormalProofEvidenceTier {
    /// Return the stable serialized evidence-tier spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FormalProof => FORMAL_PROOF_EVIDENCE_TIER,
            Self::RecordedOnly => RECORDED_ONLY_EVIDENCE_TIER,
        }
    }
}

/// BLAKE3 identity for one locally reviewable proof artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalProofArtifactIdentity {
    /// Repo-relative artifact locator.
    pub locator: String,
    /// BLAKE3 digest of the exact artifact bytes.
    pub blake3: String,
}

/// The requirement and claim classification carried by a formal-proof envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalProofClaim {
    /// Stable theorem or property identifier.
    pub theorem_or_property_id: String,
    /// Requirement or policy identifier authorizing the claim.
    pub requirement_id: String,
    /// Reviewed claim classification.
    pub claim_class: String,
}

/// The common formal-proof envelope shared by closed provider payloads.
// r[impl octet.aeneas_lean_proof_rail.provider_contract]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalProofEnvelope {
    /// Envelope schema version.
    pub schema_version: String,
    /// Provider identifier duplicated for explicit cross-checking.
    pub provider_id: String,
    /// Owning policy identifier.
    pub policy_id: String,
    /// Stable rail identifier.
    pub rail_id: String,
    /// Scoped checked-source label.
    pub checked_scope: String,
    /// Provider execution status.
    pub status: FormalProofStatus,
    /// Evidence tier assigned after validation.
    pub evidence_tier: FormalProofEvidenceTier,
    /// Requirement and theorem/property identity.
    pub claim: FormalProofClaim,
    /// BLAKE3 identity of the provider package lock.
    pub package_lock_identity: String,
    /// Source artifacts bound by the proof.
    pub source_bindings: Vec<FormalProofArtifactIdentity>,
    /// Other proof-stage artifacts bound by the proof.
    pub artifact_identities: Vec<FormalProofArtifactIdentity>,
    /// Explicit trust-boundary statements.
    pub trust_boundaries: Vec<String>,
    /// Explicit non-claim statements.
    pub non_claims: Vec<String>,
    /// Closed provider-specific payload.
    pub provider: FormalProofProvider,
}

/// Closed provider payload carried by the common envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum FormalProofProvider {
    /// An explicitly adapted, unchanged Verus-v1 receipt.
    #[serde(rename = "verus-v1")]
    VerusV1(VerusCompatibilityPayload),
    /// The typed Aeneas-to-Lean provider contract.
    #[serde(rename = "aeneas-lean")]
    AeneasLean(AeneasLeanProofPayload),
}

impl FormalProofProvider {
    /// Return the provider identifier represented by this payload.
    #[must_use]
    pub const fn provider_id(&self) -> &'static str {
        match self {
            Self::VerusV1(_) => VERUS_V1_PROVIDER_ID,
            Self::AeneasLean(_) => AENEAS_LEAN_PROVIDER_ID,
        }
    }
}

/// Compatibility payload that preserves the original Verus-v1 receipt structure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerusCompatibilityPayload {
    /// The unchanged Verus-v1 receipt being adapted.
    pub receipt: VerusProofReceipt,
}

/// Bounded process limits for Aeneas, Charon, and Lean execution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasExecutionLimits {
    /// Maximum stage execution time in milliseconds.
    pub timeout_ms: u64,
    /// Maximum captured stdout bytes per stage.
    pub stdout_max_bytes: usize,
    /// Maximum captured stderr bytes per stage.
    pub stderr_max_bytes: usize,
}

/// Effective Charon extraction policy bound by an Aeneas proof payload.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CharonExtractionPolicy {
    /// Whether Charon reported extraction errors.
    pub has_errors: bool,
    /// Whether Charon skipped Rust borrow checking.
    pub skip_borrowck: bool,
    /// Target triple used for extraction.
    pub target_triple: String,
    /// Target pointer width in bits.
    pub pointer_width_bits: u16,
    /// Target byte order.
    pub endianness: String,
    /// Reviewed MIR extraction start stage.
    pub mir_start_stage: String,
    /// Reviewed drop-handling policy.
    pub drop_policy: String,
    /// Reviewed Charon `start_from` setting.
    pub start_from: String,
    /// Exact include rules.
    pub include_rules: Vec<String>,
    /// Exact exclude rules.
    pub exclude_rules: Vec<String>,
    /// Exact opaque rules.
    pub opaque_rules: Vec<String>,
    /// Ordered transformation settings.
    pub transformation_settings: Vec<String>,
    /// Serialized LLBC format identifier.
    pub serialization_format: String,
}

/// Declared trust-input policy for Aeneas theorem admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasAssumptionPolicy {
    /// External model identifiers allowed by the reviewed profile.
    pub external_models: Vec<String>,
    /// Axiom or assumption identifiers allowed by the reviewed profile.
    pub axioms: Vec<String>,
    /// Whether every trust input must be declared before admission.
    pub requires_complete_declaration: bool,
}

/// Recorded-only semantic-foundation reference metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticReference {
    /// Exact semantic-reference repository revision.
    pub revision: String,
    /// License identity for the referenced material.
    pub license: String,
    /// Source sentinel used to identify the reviewed revision.
    pub source_sentinel: String,
    /// Must remain true because the reference is not proof authority.
    pub recorded_only: bool,
}

/// Closed Aeneas-to-Lean provider payload.
// r[impl octet.aeneas_lean_proof_rail.provider_contract]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasLeanProofPayload {
    /// Exact Aeneas release identity.
    pub aeneas_release: String,
    /// Exact Charon revision required by the Aeneas release.
    pub charon_revision: String,
    /// Exact OCaml toolchain identity.
    pub ocaml_toolchain: String,
    /// Exact Lean toolchain identity.
    pub lean_toolchain: String,
    /// Exact Aeneas Lean library identity.
    pub aeneas_lean_library: String,
    /// Reviewed safe-Rust source surface admitted by the provider.
    pub checked_source_surface: Vec<String>,
    /// Shell-free provider command argv.
    pub command_argv: Vec<String>,
    /// Bounded execution limits.
    pub limits: AeneasExecutionLimits,
    /// Explicit environment variable allowlist.
    pub environment_allowlist: Vec<String>,
    /// Effective Charon extraction contract.
    pub extraction_policy: CharonExtractionPolicy,
    /// Declared model and assumption policy.
    pub assumption_policy: AeneasAssumptionPolicy,
    /// Recorded-only semantic foundation reference.
    pub semantic_reference: SemanticReference,
}

/// Schema version for the ordered Aeneas stage-chain contract.
pub const AENEAS_STAGE_CHAIN_SCHEMA_VERSION: &str = "octet-aeneas-lean-stage-chain/v1";

const AENEAS_STAGE_COUNT: usize = 6;
const BLAKE3_HEX_LENGTH: usize = 64;

/// Ordered Aeneas pipeline stages.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasStageKind {
    /// Checked Rust source and selected item scope.
    RustSource,
    /// Charon identity, effective options, target metadata, and LLBC output.
    CharonExtraction,
    /// Aeneas identity, command configuration, and translation manifest.
    AeneasTranslation,
    /// Generated Lean definitions and imported external model inventory.
    GeneratedLean,
    /// Hand-maintained theorem source and approved statement identity.
    Theorem,
    /// Lean package/toolchain identity and final checker result.
    LeanCheck,
}

impl AeneasStageKind {
    /// Return the stable serialized stage name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RustSource => "rust_source",
            Self::CharonExtraction => "charon_extraction",
            Self::AeneasTranslation => "aeneas_translation",
            Self::GeneratedLean => "generated_lean",
            Self::Theorem => "theorem",
            Self::LeanCheck => "lean_check",
        }
    }
}

const EXPECTED_STAGE_KINDS: [AeneasStageKind; AENEAS_STAGE_COUNT] = [
    AeneasStageKind::RustSource,
    AeneasStageKind::CharonExtraction,
    AeneasStageKind::AeneasTranslation,
    AeneasStageKind::GeneratedLean,
    AeneasStageKind::Theorem,
    AeneasStageKind::LeanCheck,
];

/// Typed role for a stage-chain artifact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasStageArtifactRole {
    /// Provider profile identity.
    ProviderProfile,
    /// Checked Rust source bytes or source manifest.
    RustSource,
    /// Effective serialized Charon extraction contract.
    CharonOptions,
    /// Charon argv identity.
    CharonCommand,
    /// Emitted LLBC bytes.
    Llbc,
    /// Aeneas argv/backend identity.
    AeneasCommand,
    /// Aeneas `translation.json` manifest.
    TranslationManifest,
    /// Generated Lean definitions and file inventory.
    GeneratedLean,
    /// Explicit inventory of imported external models.
    ExternalModelInventory,
    /// Hand-maintained theorem source.
    TheoremSource,
    /// Approved theorem statement identity.
    TheoremStatement,
    /// Deterministic theorem-adequacy decision.
    TheoremAdequacy,
    /// Complete theorem trust-closure identity.
    TrustClosure,
    /// Lean package/toolchain identity.
    LeanToolchain,
    /// Aeneas Lean library identity.
    AeneasLeanLibrary,
    /// Lean checker argv identity.
    LeanCommand,
    /// Lean checker/package-lock result identity.
    CheckerResult,
    /// Aeneas package-lock identity.
    PackageLock,
}

impl AeneasStageArtifactRole {
    /// Return the stable serialized role name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderProfile => "provider_profile",
            Self::RustSource => "rust_source",
            Self::CharonOptions => "charon_options",
            Self::CharonCommand => "charon_command",
            Self::Llbc => "llbc",
            Self::AeneasCommand => "aeneas_command",
            Self::TranslationManifest => "translation_manifest",
            Self::GeneratedLean => "generated_lean",
            Self::ExternalModelInventory => "external_model_inventory",
            Self::TheoremSource => "theorem_source",
            Self::TheoremStatement => "theorem_statement",
            Self::TheoremAdequacy => "theorem_adequacy",
            Self::TrustClosure => "trust_closure",
            Self::LeanToolchain => "lean_toolchain",
            Self::AeneasLeanLibrary => "aeneas_lean_library",
            Self::LeanCommand => "lean_command",
            Self::CheckerResult => "checker_result",
            Self::PackageLock => "package_lock",
        }
    }
}

/// BLAKE3-bound typed identity for one Aeneas stage artifact.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasStageArtifactIdentity {
    /// Semantic role of the artifact in the stage chain.
    pub role: AeneasStageArtifactRole,
    /// Repo-relative artifact locator.
    pub locator: String,
    /// Lowercase BLAKE3 digest of exact artifact bytes.
    pub blake3: String,
}

/// One ordered stage with exact input/output artifact links.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasStage {
    /// Ordered stage role.
    pub kind: AeneasStageKind,
    /// Exact artifacts consumed from the previous stage.
    pub input_artifacts: Vec<AeneasStageArtifactIdentity>,
    /// Exact artifacts emitted for the next stage.
    pub output_artifacts: Vec<AeneasStageArtifactIdentity>,
}

/// Complete ordered, hash-linked Aeneas stage chain.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasStageChain {
    /// Stage-chain schema version.
    pub schema_version: String,
    /// BLAKE3-bound provider profile identity.
    pub provider_profile: AeneasStageArtifactIdentity,
    /// BLAKE3-bound package-lock identity.
    pub package_lock: AeneasStageArtifactIdentity,
    /// Exactly six ordered stages from Rust source to Lean result.
    pub stages: Vec<AeneasStage>,
}

/// Return the stage names in their recorded order without performing I/O.
#[must_use]
pub fn aeneas_stage_chain_stage_order(chain: &AeneasStageChain) -> Vec<String> {
    chain.stages.iter().map(|stage| stage.kind.as_str().to_owned()).collect()
}

/// Validate ordered stage roles, BLAKE3 identities, and adjacent stage links.
#[must_use]
pub fn validate_aeneas_stage_chain(chain: &AeneasStageChain) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_exact(
        &mut diagnostics,
        "stage_chain.schema_version",
        AENEAS_STAGE_CHAIN_SCHEMA_VERSION,
        &chain.schema_version,
    );
    validate_stage_artifact(
        "stage_chain.provider_profile",
        &chain.provider_profile,
        AeneasStageArtifactRole::ProviderProfile,
        &mut diagnostics,
    );
    validate_stage_artifact(
        "stage_chain.package_lock",
        &chain.package_lock,
        AeneasStageArtifactRole::PackageLock,
        &mut diagnostics,
    );
    if chain.provider_profile == chain.package_lock {
        diagnostics.push("stage_chain provider_profile and package_lock must be distinct".to_owned());
    }
    if chain.stages.len() != AENEAS_STAGE_COUNT {
        diagnostics
            .push(format!("stage_chain.stages expected {AENEAS_STAGE_COUNT} stages but got {}", chain.stages.len()));
    }

    let mut output_identities = BTreeSet::new();
    for (index, stage) in chain.stages.iter().enumerate() {
        let stage_prefix = format!("stage_chain.stages[{index}]");
        let expected_kind = EXPECTED_STAGE_KINDS.get(index).copied();
        if expected_kind != Some(stage.kind) {
            let expected_name = expected_kind.map_or("missing", AeneasStageKind::as_str);
            diagnostics
                .push(format!("{stage_prefix}.kind expected `{expected_name}` but got `{}`", stage.kind.as_str()));
        }
        if index != 0 || !stage.input_artifacts.is_empty() {
            validate_stage_artifact_list(
                &format!("{stage_prefix}.input_artifacts"),
                &stage.input_artifacts,
                &mut diagnostics,
            );
        }
        validate_stage_artifact_list(
            &format!("{stage_prefix}.output_artifacts"),
            &stage.output_artifacts,
            &mut diagnostics,
        );
        let expected_roles = expected_output_roles(stage.kind);
        let actual_roles = stage.output_artifacts.iter().map(|artifact| artifact.role).collect::<Vec<_>>();
        if actual_roles != expected_roles {
            diagnostics.push(format!(
                "{stage_prefix}.output_artifacts roles do not match `{}` stage contract",
                stage.kind.as_str()
            ));
        }
        for artifact in &stage.output_artifacts {
            if !output_identities.insert((artifact.role, artifact.locator.clone(), artifact.blake3.clone())) {
                diagnostics.push(format!(
                    "{stage_prefix}.output_artifacts duplicates `{}` at `{}`",
                    artifact.role.as_str(),
                    artifact.locator
                ));
            }
        }
        if index == 0 {
            if !stage.input_artifacts.is_empty() {
                diagnostics.push(format!("{stage_prefix}.input_artifacts must be empty for the root stage"));
            }
        } else if let Some(previous_stage) = chain.stages.get(index - 1)
            && stage.input_artifacts != previous_stage.output_artifacts
        {
            diagnostics.push(format!(
                "{stage_prefix}.input_artifacts do not exactly match previous `{}` outputs",
                previous_stage.kind.as_str()
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn expected_output_roles(kind: AeneasStageKind) -> Vec<AeneasStageArtifactRole> {
    match kind {
        AeneasStageKind::RustSource => vec![AeneasStageArtifactRole::RustSource],
        AeneasStageKind::CharonExtraction => vec![
            AeneasStageArtifactRole::CharonOptions,
            AeneasStageArtifactRole::CharonCommand,
            AeneasStageArtifactRole::Llbc,
        ],
        AeneasStageKind::AeneasTranslation => vec![
            AeneasStageArtifactRole::AeneasCommand,
            AeneasStageArtifactRole::TranslationManifest,
        ],
        AeneasStageKind::GeneratedLean => vec![
            AeneasStageArtifactRole::GeneratedLean,
            AeneasStageArtifactRole::ExternalModelInventory,
        ],
        AeneasStageKind::Theorem => vec![
            AeneasStageArtifactRole::TheoremSource,
            AeneasStageArtifactRole::TheoremStatement,
            AeneasStageArtifactRole::TheoremAdequacy,
            AeneasStageArtifactRole::TrustClosure,
        ],
        AeneasStageKind::LeanCheck => vec![
            AeneasStageArtifactRole::LeanToolchain,
            AeneasStageArtifactRole::AeneasLeanLibrary,
            AeneasStageArtifactRole::LeanCommand,
            AeneasStageArtifactRole::CheckerResult,
        ],
    }
}

fn validate_stage_artifact_list(field: &str, artifacts: &[AeneasStageArtifactIdentity], diagnostics: &mut Vec<String>) {
    if artifacts.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    let mut identities = BTreeSet::new();
    for artifact in artifacts {
        validate_stage_artifact(field, artifact, artifact.role, diagnostics);
        if !identities.insert((artifact.role, artifact.locator.clone(), artifact.blake3.clone())) {
            diagnostics.push(format!("{field} contains duplicate `{}`", artifact.role.as_str()));
        }
    }
}

fn validate_stage_artifact(
    field: &str,
    artifact: &AeneasStageArtifactIdentity,
    expected_role: AeneasStageArtifactRole,
    diagnostics: &mut Vec<String>,
) {
    if artifact.role != expected_role {
        diagnostics.push(format!(
            "{field}.role expected `{}` but got `{}`",
            expected_role.as_str(),
            artifact.role.as_str()
        ));
    }
    push_non_empty(diagnostics, &format!("{field}.locator"), &artifact.locator);
    if !is_blake3_hex(&artifact.blake3) {
        diagnostics.push(format!("{field}.blake3 must be a lowercase 64-character BLAKE3 hex digest"));
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH
        && value == value.to_ascii_lowercase()
        && value.chars().all(|character| character.is_ascii_hexdigit())
}

/// Validate the common envelope and its closed provider payload.
#[must_use]
pub fn validate_formal_proof_envelope(envelope: &FormalProofEnvelope) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_exact(&mut diagnostics, "schema_version", FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION, &envelope.schema_version);
    push_non_empty(&mut diagnostics, "provider_id", &envelope.provider_id);
    push_non_empty(&mut diagnostics, "policy_id", &envelope.policy_id);
    push_non_empty(&mut diagnostics, "rail_id", &envelope.rail_id);
    push_non_empty(&mut diagnostics, "checked_scope", &envelope.checked_scope);
    push_non_empty(&mut diagnostics, "claim.theorem_or_property_id", &envelope.claim.theorem_or_property_id);
    push_non_empty(&mut diagnostics, "claim.requirement_id", &envelope.claim.requirement_id);
    push_non_empty(&mut diagnostics, "claim.claim_class", &envelope.claim.claim_class);
    push_non_empty(&mut diagnostics, "package_lock_identity", &envelope.package_lock_identity);
    push_artifact_identities(&mut diagnostics, "source_bindings", &envelope.source_bindings);
    push_artifact_identities(&mut diagnostics, "artifact_identities", &envelope.artifact_identities);
    push_non_empty_strings(&mut diagnostics, "trust_boundaries", &envelope.trust_boundaries);
    push_non_empty_strings(&mut diagnostics, "non_claims", &envelope.non_claims);
    if envelope.provider_id != envelope.provider.provider_id() {
        diagnostics.push(format!(
            "provider_id `{}` does not match closed payload provider `{}`",
            envelope.provider_id,
            envelope.provider.provider_id()
        ));
    }
    if envelope.evidence_tier == FormalProofEvidenceTier::FormalProof && envelope.status != FormalProofStatus::Passed {
        diagnostics.push("formal_proof evidence requires passed provider status".to_owned());
    }
    if let FormalProofProvider::AeneasLean(payload) = &envelope.provider {
        diagnostics.extend(validate_aeneas_lean_payload(payload));
        for required in [
            AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS,
            AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS,
            AENEAS_NON_CLAIM_WHOLE_PROGRAM,
        ] {
            if !envelope.non_claims.iter().any(|claim| claim == required) {
                diagnostics.push(format!("non_claims missing required Aeneas boundary `{required}`"));
            }
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Validate the Aeneas provider payload without process or filesystem access.
#[must_use]
pub fn validate_aeneas_lean_payload(payload: &AeneasLeanProofPayload) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for field in REQUIRED_AENEAS_FIELDS {
        let value = match *field {
            "aeneas_release" => &payload.aeneas_release,
            "charon_revision" => &payload.charon_revision,
            "ocaml_toolchain" => &payload.ocaml_toolchain,
            "lean_toolchain" => &payload.lean_toolchain,
            "aeneas_lean_library" => &payload.aeneas_lean_library,
            "target_triple" => &payload.extraction_policy.target_triple,
            "mir_start_stage" => &payload.extraction_policy.mir_start_stage,
            "drop_policy" => &payload.extraction_policy.drop_policy,
            "start_from" => &payload.extraction_policy.start_from,
            "serialization_format" => &payload.extraction_policy.serialization_format,
            _ => unreachable!("required Aeneas field list contains an unknown field"),
        };
        push_non_empty(&mut diagnostics, field, value);
    }
    push_non_empty_strings(&mut diagnostics, "checked_source_surface", &payload.checked_source_surface);
    push_non_empty_strings(&mut diagnostics, "command_argv", &payload.command_argv);
    push_non_empty_strings(&mut diagnostics, "environment_allowlist", &payload.environment_allowlist);
    if payload.limits.timeout_ms == 0 {
        diagnostics.push("limits.timeout_ms must be greater than zero".to_owned());
    }
    if payload.limits.stdout_max_bytes == 0 {
        diagnostics.push("limits.stdout_max_bytes must be greater than zero".to_owned());
    }
    if payload.limits.stderr_max_bytes == 0 {
        diagnostics.push("limits.stderr_max_bytes must be greater than zero".to_owned());
    }
    if payload.extraction_policy.has_errors {
        diagnostics.push("extraction_policy.has_errors must be false".to_owned());
    }
    if payload.extraction_policy.skip_borrowck {
        diagnostics.push("extraction_policy.skip_borrowck must be false".to_owned());
    }
    if payload.extraction_policy.pointer_width_bits == 0 {
        diagnostics.push("extraction_policy.pointer_width_bits must be greater than zero".to_owned());
    }
    if !matches!(payload.extraction_policy.endianness.as_str(), LITTLE_ENDIAN | BIG_ENDIAN) {
        diagnostics.push(format!("extraction_policy.endianness must be `{LITTLE_ENDIAN}` or `{BIG_ENDIAN}`"));
    }
    if !payload.assumption_policy.requires_complete_declaration {
        diagnostics.push("assumption_policy.requires_complete_declaration must be true".to_owned());
    }
    push_no_blank_strings(
        &mut diagnostics,
        "assumption_policy.external_models",
        &payload.assumption_policy.external_models,
    );
    push_no_blank_strings(&mut diagnostics, "assumption_policy.axioms", &payload.assumption_policy.axioms);
    push_non_empty(&mut diagnostics, "semantic_reference.revision", &payload.semantic_reference.revision);
    push_non_empty(&mut diagnostics, "semantic_reference.license", &payload.semantic_reference.license);
    push_non_empty(&mut diagnostics, "semantic_reference.source_sentinel", &payload.semantic_reference.source_sentinel);
    if !payload.semantic_reference.recorded_only {
        diagnostics.push("semantic_reference.recorded_only must be true".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// r[impl octet.aeneas_lean_proof_rail.verus_compatibility]
/// Adapt a Verus-v1 receipt without changing or reserializing its receipt schema.
///
/// # Errors
///
/// Returns an error when the existing receipt uses an unknown status spelling.
pub fn adapt_verus_v1_receipt(receipt: VerusProofReceipt) -> Result<FormalProofEnvelope, String> {
    let status = match receipt.status.as_str() {
        PROOF_STATUS_PASSED => FormalProofStatus::Passed,
        PROOF_STATUS_FAILED => FormalProofStatus::Failed,
        status => return Err(format!("unsupported Verus-v1 receipt status `{status}`")),
    };
    let evidence_tier = if status == FormalProofStatus::Passed {
        FormalProofEvidenceTier::FormalProof
    } else {
        FormalProofEvidenceTier::RecordedOnly
    };
    let rail_id = format!("{}#rail-{}", receipt.policy_id, receipt.rail_index);
    let source_bindings = receipt
        .source_bindings
        .iter()
        .map(|binding| FormalProofArtifactIdentity {
            locator: binding.path.clone(),
            blake3: binding.blake3.clone(),
        })
        .collect::<Vec<_>>();
    let artifact_identities = receipt
        .proof_bindings
        .iter()
        .map(|binding| FormalProofArtifactIdentity {
            locator: binding.path.clone(),
            blake3: binding.blake3.clone(),
        })
        .chain([
            FormalProofArtifactIdentity {
                locator: receipt.stdout_artifact.locator.clone(),
                blake3: receipt.stdout_artifact.blake3.clone(),
            },
            FormalProofArtifactIdentity {
                locator: receipt.stderr_artifact.locator.clone(),
                blake3: receipt.stderr_artifact.blake3.clone(),
            },
        ])
        .collect::<Vec<_>>();
    Ok(FormalProofEnvelope {
        schema_version: FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION.to_owned(),
        provider_id: VERUS_V1_PROVIDER_ID.to_owned(),
        policy_id: receipt.policy_id.clone(),
        rail_id: rail_id.clone(),
        checked_scope: receipt.checked_scope.clone(),
        status,
        evidence_tier,
        claim: FormalProofClaim {
            theorem_or_property_id: rail_id,
            requirement_id: receipt.policy_id.clone(),
            claim_class: VERUS_COMPATIBILITY_CLAIM_CLASS.to_owned(),
        },
        package_lock_identity: receipt.package_lock_blake3.clone(),
        source_bindings,
        artifact_identities,
        trust_boundaries: receipt.trust_boundaries.clone(),
        non_claims: receipt.non_claims.clone(),
        provider: FormalProofProvider::VerusV1(VerusCompatibilityPayload { receipt }),
    })
}

fn push_exact(diagnostics: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if actual != expected {
        diagnostics.push(format!("{field} expected `{expected}` but got `{actual}`"));
    }
}

fn push_non_empty(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
    }
}

fn push_non_empty_strings(diagnostics: &mut Vec<String>, field: &str, values: &[String]) {
    if values.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    push_no_blank_strings(diagnostics, field, values);
}

fn push_no_blank_strings(diagnostics: &mut Vec<String>, field: &str, values: &[String]) {
    for value in values {
        if value.trim().is_empty() {
            diagnostics.push(format!("{field} contains an empty value"));
        }
    }
}

fn push_artifact_identities(diagnostics: &mut Vec<String>, field: &str, identities: &[FormalProofArtifactIdentity]) {
    if identities.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    for identity in identities {
        push_non_empty(diagnostics, &format!("{field}.locator"), &identity.locator);
        push_non_empty(diagnostics, &format!("{field}.blake3"), &identity.blake3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::VerusLogArtifact;
    use crate::proof::VerusProofBinding;
    use crate::proof::VerusSolverIdentity;
    use crate::proof::VerusToolchainIdentity;
    use crate::proof::VerusVerifierIdentity;

    const TEST_TIMEOUT_MS: u64 = 30_000;
    const TEST_LOG_MAX_BYTES: usize = 4_096;
    const TEST_POINTER_WIDTH_BITS: u16 = 64;
    const TEST_PROVIDER_RELEASE: &str = "aeneas@0.1.0";
    const TEST_CHARON_REVISION: &str = "cb50ff16b9f1066b8a97dc06da704de2da2fa41c";
    const TEST_SOURCE_PATH: &str = "src/verified/addition.rs";
    const TEST_NON_CLAIMS: &[&str] = &[
        AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS,
        AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS,
        AENEAS_NON_CLAIM_WHOLE_PROGRAM,
    ];

    // r[verify octet.aeneas_lean_proof_rail.provider_contract]
    #[test]
    fn aeneas_payload_and_envelope_accept_complete_typed_contract() {
        let envelope = valid_aeneas_envelope();
        let diagnostics = validate_formal_proof_envelope(&envelope);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        let rendered = serde_json::to_string(&envelope).expect("serialize complete envelope");
        let parsed: FormalProofEnvelope = serde_json::from_str(&rendered).expect("parse complete envelope");
        assert_eq!(parsed, envelope);
    }

    // r[verify octet.aeneas_lean_proof_rail.provider_contract]
    #[test]
    fn aeneas_payload_rejects_weak_contract_and_unknown_fields() {
        let mut envelope = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut envelope.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.extraction_policy.skip_borrowck = true;
        payload.semantic_reference.recorded_only = false;
        payload.command_argv.clear();
        envelope.non_claims.retain(|claim| *claim != AENEAS_NON_CLAIM_WHOLE_PROGRAM);

        let diagnostics = validate_formal_proof_envelope(&envelope);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("skip_borrowck")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("command_argv")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program")));

        let rendered = serde_json::to_string(&valid_aeneas_payload()).expect("serialize payload");
        let with_unknown_field = rendered.trim_end_matches('}').to_owned() + ",\"unknown\":true}";
        assert!(
            serde_json::from_str::<AeneasLeanProofPayload>(&with_unknown_field).is_err(),
            "closed Aeneas payload must reject unknown fields"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    #[test]
    fn aeneas_stage_chain_accepts_ordered_hash_linked_pipeline() {
        let chain = valid_stage_chain();

        let diagnostics = validate_aeneas_stage_chain(&chain);
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(aeneas_stage_chain_stage_order(&chain), vec![
            "rust_source",
            "charon_extraction",
            "aeneas_translation",
            "generated_lean",
            "theorem",
            "lean_check",
        ]);
        let rendered = serde_json::to_string(&chain).expect("serialize stage chain");
        let parsed: AeneasStageChain = serde_json::from_str(&rendered).expect("parse stage chain");
        assert_eq!(parsed, chain);
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    #[test]
    fn aeneas_stage_chain_rejects_stale_swapped_duplicate_and_malformed_links() {
        let mut chain = valid_stage_chain();
        chain.stages[2].input_artifacts[0].blake3 = "stale".to_owned();
        chain.stages[1].output_artifacts[1] = chain.stages[1].output_artifacts[0].clone();
        chain.stages[3].output_artifacts.swap(0, 1);
        chain.stages[4].input_artifacts.reverse();
        chain.stages[5].output_artifacts[0].blake3 = "not-a-blake3-digest".to_owned();

        let diagnostics = validate_aeneas_stage_chain(&chain);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("previous")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("roles")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("BLAKE3")));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("duplicate") || diagnostic.contains("match previous"))
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.verus_compatibility]
    #[test]
    fn verus_adapter_preserves_receipt_and_uses_common_identity() {
        let receipt = valid_verus_receipt();
        let original = receipt.clone();
        let original_text = serde_json::to_string(&original).expect("serialize original Verus receipt");
        let envelope = adapt_verus_v1_receipt(receipt).expect("adapt Verus receipt");

        assert_eq!(envelope.provider_id, VERUS_V1_PROVIDER_ID);
        assert_eq!(envelope.status, FormalProofStatus::Passed);
        assert_eq!(envelope.evidence_tier, FormalProofEvidenceTier::FormalProof);
        assert_eq!(envelope.package_lock_identity, original.package_lock_blake3);
        let FormalProofProvider::VerusV1(payload) = envelope.provider else {
            panic!("adapter must retain Verus provider payload");
        };
        assert_eq!(payload.receipt, original);
        assert_eq!(serde_json::to_string(&payload.receipt).expect("serialize adapted Verus receipt"), original_text);
    }

    // r[verify octet.aeneas_lean_proof_rail.verus_compatibility]
    #[test]
    fn verus_adapter_rejects_unknown_receipt_status_without_reinterpretation() {
        let mut receipt = valid_verus_receipt();
        receipt.status = "proved".to_owned();

        let error = adapt_verus_v1_receipt(receipt).expect_err("unknown status must fail closed");

        assert!(error.contains("unsupported Verus-v1 receipt status"));
    }

    fn valid_aeneas_payload() -> AeneasLeanProofPayload {
        AeneasLeanProofPayload {
            aeneas_release: TEST_PROVIDER_RELEASE.to_owned(),
            charon_revision: TEST_CHARON_REVISION.to_owned(),
            ocaml_toolchain: "ocaml@5.2.0".to_owned(),
            lean_toolchain: "lean@4.19.0".to_owned(),
            aeneas_lean_library: "aeneas-lean@0.1.0".to_owned(),
            checked_source_surface: vec!["safe sequential Rust".to_owned()],
            command_argv: vec!["aeneas-lean".to_owned(), "--emit-json".to_owned()],
            limits: AeneasExecutionLimits {
                timeout_ms: TEST_TIMEOUT_MS,
                stdout_max_bytes: TEST_LOG_MAX_BYTES,
                stderr_max_bytes: TEST_LOG_MAX_BYTES,
            },
            environment_allowlist: vec!["PATH".to_owned()],
            extraction_policy: CharonExtractionPolicy {
                has_errors: false,
                skip_borrowck: false,
                target_triple: "x86_64-unknown-linux-gnu".to_owned(),
                pointer_width_bits: TEST_POINTER_WIDTH_BITS,
                endianness: LITTLE_ENDIAN.to_owned(),
                mir_start_stage: "borrowck".to_owned(),
                drop_policy: "include-drops".to_owned(),
                start_from: "crate".to_owned(),
                include_rules: vec![TEST_SOURCE_PATH.to_owned()],
                exclude_rules: Vec::new(),
                opaque_rules: Vec::new(),
                transformation_settings: vec!["preserve-spans".to_owned()],
                serialization_format: "llbc-json-v1".to_owned(),
            },
            assumption_policy: AeneasAssumptionPolicy {
                external_models: vec!["none".to_owned()],
                axioms: vec!["none".to_owned()],
                requires_complete_declaration: true,
            },
            semantic_reference: SemanticReference {
                revision: "mechanized-llbc@deadbeef".to_owned(),
                license: "MIT".to_owned(),
                source_sentinel: "README:semantic-foundation".to_owned(),
                recorded_only: true,
            },
        }
    }

    fn valid_stage_chain() -> AeneasStageChain {
        let source = stage_artifact(AeneasStageArtifactRole::RustSource, "src/verified/addition.rs");
        let charon_options = stage_artifact(AeneasStageArtifactRole::CharonOptions, "stages/charon-options.json");
        let charon_command = stage_artifact(AeneasStageArtifactRole::CharonCommand, "stages/charon.argv");
        let llbc = stage_artifact(AeneasStageArtifactRole::Llbc, "stages/addition.llbc");
        let aeneas_command = stage_artifact(AeneasStageArtifactRole::AeneasCommand, "stages/aeneas.argv");
        let translation = stage_artifact(AeneasStageArtifactRole::TranslationManifest, "stages/translation.json");
        let generated_lean = stage_artifact(AeneasStageArtifactRole::GeneratedLean, "generated/Add.lean");
        let external_models =
            stage_artifact(AeneasStageArtifactRole::ExternalModelInventory, "generated/external-models.json");
        let theorem_source = stage_artifact(AeneasStageArtifactRole::TheoremSource, "theorems/addition.lean");
        let theorem_statement =
            stage_artifact(AeneasStageArtifactRole::TheoremStatement, "theorems/addition.statement");
        let theorem_adequacy =
            stage_artifact(AeneasStageArtifactRole::TheoremAdequacy, "theorems/addition.adequacy.json");
        let trust_closure = stage_artifact(AeneasStageArtifactRole::TrustClosure, "theorems/addition.trust.json");
        let lean_toolchain = stage_artifact(AeneasStageArtifactRole::LeanToolchain, "lean-toolchain");
        let aeneas_library = stage_artifact(AeneasStageArtifactRole::AeneasLeanLibrary, "lake-manifest.json");
        let lean_command = stage_artifact(AeneasStageArtifactRole::LeanCommand, "stages/lean.argv");
        let checker_result = stage_artifact(AeneasStageArtifactRole::CheckerResult, "stages/lean-check.json");
        AeneasStageChain {
            schema_version: AENEAS_STAGE_CHAIN_SCHEMA_VERSION.to_owned(),
            provider_profile: stage_artifact(AeneasStageArtifactRole::ProviderProfile, "profile.json"),
            package_lock: stage_artifact(AeneasStageArtifactRole::PackageLock, "package-lock.json"),
            stages: vec![
                AeneasStage {
                    kind: AeneasStageKind::RustSource,
                    input_artifacts: Vec::new(),
                    output_artifacts: vec![source],
                },
                AeneasStage {
                    kind: AeneasStageKind::CharonExtraction,
                    input_artifacts: vec![stage_artifact(
                        AeneasStageArtifactRole::RustSource,
                        "src/verified/addition.rs",
                    )],
                    output_artifacts: vec![charon_options, charon_command, llbc],
                },
                AeneasStage {
                    kind: AeneasStageKind::AeneasTranslation,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::CharonOptions, "stages/charon-options.json"),
                        stage_artifact(AeneasStageArtifactRole::CharonCommand, "stages/charon.argv"),
                        stage_artifact(AeneasStageArtifactRole::Llbc, "stages/addition.llbc"),
                    ],
                    output_artifacts: vec![aeneas_command, translation],
                },
                AeneasStage {
                    kind: AeneasStageKind::GeneratedLean,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::AeneasCommand, "stages/aeneas.argv"),
                        stage_artifact(AeneasStageArtifactRole::TranslationManifest, "stages/translation.json"),
                    ],
                    output_artifacts: vec![generated_lean, external_models],
                },
                AeneasStage {
                    kind: AeneasStageKind::Theorem,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::GeneratedLean, "generated/Add.lean"),
                        stage_artifact(
                            AeneasStageArtifactRole::ExternalModelInventory,
                            "generated/external-models.json",
                        ),
                    ],
                    output_artifacts: vec![theorem_source, theorem_statement, theorem_adequacy, trust_closure],
                },
                AeneasStage {
                    kind: AeneasStageKind::LeanCheck,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::TheoremSource, "theorems/addition.lean"),
                        stage_artifact(AeneasStageArtifactRole::TheoremStatement, "theorems/addition.statement"),
                        stage_artifact(AeneasStageArtifactRole::TheoremAdequacy, "theorems/addition.adequacy.json"),
                        stage_artifact(AeneasStageArtifactRole::TrustClosure, "theorems/addition.trust.json"),
                    ],
                    output_artifacts: vec![lean_toolchain, aeneas_library, lean_command, checker_result],
                },
            ],
        }
    }

    fn stage_artifact(role: AeneasStageArtifactRole, locator: &str) -> AeneasStageArtifactIdentity {
        AeneasStageArtifactIdentity {
            role,
            locator: locator.to_owned(),
            blake3: crate::proof_artifact::blake3_hex_text(locator),
        }
    }

    fn valid_aeneas_envelope() -> FormalProofEnvelope {
        FormalProofEnvelope {
            schema_version: FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION.to_owned(),
            provider_id: AENEAS_LEAN_PROVIDER_ID.to_owned(),
            policy_id: "octet.aeneas_lean_proof_rail.fixture".to_owned(),
            rail_id: "octet.aeneas_lean_proof_rail.fixture#rail-0".to_owned(),
            checked_scope: "formal-proof:fixture-only:aeneas#rail-0".to_owned(),
            status: FormalProofStatus::Passed,
            evidence_tier: FormalProofEvidenceTier::FormalProof,
            claim: FormalProofClaim {
                theorem_or_property_id: "aeneas.fixture.safe_add".to_owned(),
                requirement_id: "octet.aeneas_lean_proof_rail.provider_contract".to_owned(),
                claim_class: "named-theorem-over-generated-model".to_owned(),
            },
            package_lock_identity: "b3:package-lock".to_owned(),
            source_bindings: vec![artifact(TEST_SOURCE_PATH)],
            artifact_identities: vec![artifact("proofs/aeneas.translation.json")],
            trust_boundaries: vec!["fixture-only provider contract".to_owned()],
            non_claims: TEST_NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
            provider: FormalProofProvider::AeneasLean(valid_aeneas_payload()),
        }
    }

    fn valid_verus_receipt() -> VerusProofReceipt {
        VerusProofReceipt {
            schema_version: "octet-verus-proof-receipt/v1".to_owned(),
            policy_id: "octet.verus.fixture".to_owned(),
            rail_index: 0,
            backend: "verus".to_owned(),
            verifier_mode: "fixture".to_owned(),
            producer_id: "verus@fixture".to_owned(),
            checker_id: "octet-standards".to_owned(),
            package_lock_blake3: "b3:package-lock".to_owned(),
            verifier_identity: VerusVerifierIdentity {
                verifier_id: "verus@fixture".to_owned(),
                verifier_digest: "b3:verifier".to_owned(),
                verifier_binary_path: "verus".to_owned(),
                verifier_binary_digest: "b3:binary".to_owned(),
                ..VerusVerifierIdentity::default()
            },
            toolchain_identity: VerusToolchainIdentity {
                toolchain_id: "rust@fixture".to_owned(),
                toolchain_digest: "b3:toolchain".to_owned(),
            },
            solver_identity: VerusSolverIdentity::default(),
            command_blake3: "b3:command".to_owned(),
            source_bindings: vec![VerusProofBinding {
                path: TEST_SOURCE_PATH.to_owned(),
                blake3: "b3:source".to_owned(),
            }],
            proof_bindings: vec![VerusProofBinding {
                path: "verus/proof.rs".to_owned(),
                blake3: "b3:proof".to_owned(),
            }],
            trust_boundaries: vec!["fixture-only".to_owned()],
            checked_scope: "formal-proof:fixture-only:verus#rail-0".to_owned(),
            status: PROOF_STATUS_PASSED.to_owned(),
            pass_signal: "proof-result: pass".to_owned(),
            proof_artifact_identity: "proofs/verus.receipt".to_owned(),
            stdout_artifact: VerusLogArtifact {
                locator: "proofs/verus.stdout".to_owned(),
                blake3: "b3:stdout".to_owned(),
                bytes: 1,
                truncated: false,
            },
            stderr_artifact: VerusLogArtifact {
                locator: "proofs/verus.stderr".to_owned(),
                blake3: "b3:stderr".to_owned(),
                bytes: 0,
                truncated: false,
            },
            non_claims: vec!["not whole-program correctness".to_owned()],
        }
    }

    fn artifact(locator: &str) -> FormalProofArtifactIdentity {
        FormalProofArtifactIdentity {
            locator: locator.to_owned(),
            blake3: "b3:artifact".to_owned(),
        }
    }
}
