use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use wasmparser::Encoding;
use wasmparser::Operator;
use wasmparser::Parser;
use wasmparser::Payload;
use wasmparser::Validator;
use wasmparser::WasmFeatures;

pub(crate) const REGISTRY_SCHEMA_VERSION: &str = "octet-wasm-artifact-profiles/v1";
pub(crate) const ENVELOPE_SCHEMA_VERSION: &str = "octet-wasm-artifact-analysis/v1";
pub(crate) const INPUT_ARTIFACT_NAME: &str = "wasm-artifact.bin";
pub(crate) const RECORDED_ONLY_ROLE: &str = "recorded_only";
pub(crate) const REQUIRED_NON_CLAIM: &str =
    "Wasm artifact evidence records bounded static facts and does not prove execution correctness";

const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const COHORT_IDENTITY_DOMAIN: &[u8] = b"octet-wasm-tools-cohort/v1\0";
const INTERFACE_IDENTITY_DOMAIN: &[u8] = b"octet-wasm-interface/v1\0";
const MALFORMED_ISSUE: &str = "malformed-artifact";
const DISALLOWED_KIND_ISSUE: &str = "disallowed-artifact-kind";
const DISALLOWED_PROPOSAL_ISSUE: &str = "disallowed-proposal";
const DISALLOWED_IMPORT_ISSUE: &str = "disallowed-import";
const DISALLOWED_EXPORT_ISSUE: &str = "disallowed-export";
const DISALLOWED_WORLD_ISSUE: &str = "disallowed-world";
const PRODUCER_METADATA_ISSUE: &str = "producer-metadata";
const RESOURCE_LIMIT_ISSUE: &str = "resource-limit";
const MEMORY_GROWTH_ISSUE: &str = "memory-growth";
const START_FUNCTION_ISSUE: &str = "start-function";
const DUPLICATE_METADATA_ISSUE: &str = "duplicate-custom-section";
const ERROR_SEVERITY: &str = "error";
const MODULE_KIND: &str = "module";
const COMPONENT_KIND: &str = "component";
const UNKNOWN_KIND: &str = "unknown";
const WASM1_PROPOSAL: &str = "wasm1";
const WASM2_PROPOSAL: &str = "wasm2";
const WASM3_PROPOSAL: &str = "wasm3";
const COMPONENT_MODEL_PROPOSAL: &str = "component-model";
const PRODUCERS_SECTION: &str = "producers";
const COMPONENT_WORLD_SEPARATOR: char = '/';
const STANDALONE_HANDOFF: &str = "standalone";
const MANTLE_HANDOFF: &str = "mantle-exact-byte";
const EXTERNAL_HANDOFF: &str = "external-orchestrator";
const MANTLE_PARENT_PREFIX: &str = "mantle:";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmProfileRegistry {
    pub(crate) schema_version: String,
    pub(crate) cohort: WasmToolsCohort,
    pub(crate) profiles: Vec<WasmArtifactProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmToolsCohort {
    pub(crate) cohort_id: String,
    pub(crate) identity: String,
    pub(crate) members: Vec<WasmToolsCohortMember>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct WasmToolsCohortMember {
    pub(crate) crate_name: String,
    pub(crate) version: String,
    pub(crate) role: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactProfile {
    pub(crate) profile_id: String,
    pub(crate) summary: String,
    pub(crate) allowed_artifact_kinds: Vec<String>,
    pub(crate) allowed_proposals: Vec<String>,
    pub(crate) allowed_worlds: Vec<String>,
    pub(crate) allowed_imports: Vec<String>,
    pub(crate) allow_any_exports: bool,
    pub(crate) resource_limits: WasmResourceLimits,
    pub(crate) allow_memory_grow: bool,
    pub(crate) allow_start: bool,
    pub(crate) duplicate_custom_sections: String,
    pub(crate) producer_metadata: String,
    pub(crate) severity: WasmFindingSeverities,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmResourceLimits {
    pub(crate) max_artifact_bytes: u64,
    pub(crate) max_imports: u64,
    pub(crate) max_exports: u64,
    pub(crate) max_memories: u64,
    pub(crate) max_tables: u64,
    pub(crate) max_functions: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmFindingSeverities {
    pub(crate) malformed: String,
    pub(crate) disallowed_kind: String,
    pub(crate) disallowed_import: String,
    pub(crate) disallowed_world: String,
    pub(crate) resource_limit: String,
    pub(crate) growth: String,
    pub(crate) start: String,
    pub(crate) duplicate_metadata: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct WasmInterfaceItem {
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) kind: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactFacts {
    pub(crate) artifact_kind: String,
    pub(crate) proposals: Vec<String>,
    pub(crate) imports: Vec<WasmInterfaceItem>,
    pub(crate) exports: Vec<WasmInterfaceItem>,
    pub(crate) worlds: Vec<String>,
    pub(crate) memory_count: u64,
    pub(crate) table_count: u64,
    pub(crate) function_count: u64,
    pub(crate) memory_grow_count: u64,
    pub(crate) has_start: bool,
    pub(crate) custom_sections: Vec<String>,
    pub(crate) producer_sections: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactFinding {
    pub(crate) issue_code: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) byte_offset: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactReceiptDetails {
    pub(crate) profile_id: String,
    pub(crate) profile_identity: String,
    pub(crate) registry_identity: String,
    pub(crate) cohort_id: String,
    pub(crate) cohort_identity: String,
    pub(crate) exact_artifact_identity: String,
    pub(crate) interface_identity: String,
    pub(crate) exact_identity_role: String,
    pub(crate) interface_identity_role: String,
    pub(crate) input_artifact: String,
    pub(crate) handoff_role: String,
    pub(crate) verification_role: String,
    pub(crate) facts: WasmArtifactFacts,
    pub(crate) findings: Vec<WasmArtifactFinding>,
    pub(crate) sarif: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactEnvelope {
    pub(crate) schema_version: String,
    pub(crate) details: WasmArtifactReceiptDetails,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WasmEvaluationStatus {
    Passed,
    ManualReview,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WasmEnvelopeEvaluation {
    pub(crate) details: Option<WasmArtifactReceiptDetails>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) status: WasmEvaluationStatus,
}

#[derive(Default)]
struct FactExtraction {
    facts: WasmArtifactFacts,
    parse_diagnostics: Vec<ParseDiagnostic>,
}

struct ParseDiagnostic {
    issue_code: &'static str,
    message: String,
    byte_offset: Option<u64>,
}

pub(crate) fn parse_registry(text: &str) -> Result<WasmProfileRegistry, String> {
    serde_json::from_str(text).map_err(|error| format!("parse Wasm artifact profile registry: {error}"))
}

pub(crate) fn validate_registry(registry: &WasmProfileRegistry) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if registry.schema_version != REGISTRY_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Wasm profile registry schema `{}`; expected `{REGISTRY_SCHEMA_VERSION}`",
            registry.schema_version
        ));
    }
    let computed_cohort = cohort_identity(&registry.cohort.members);
    if registry.cohort.identity != computed_cohort {
        diagnostics.push(format!(
            "stale wasm-tools cohort identity; expected `{}`, observed `{computed_cohort}`",
            registry.cohort.identity
        ));
    }
    if registry.cohort.cohort_id.trim().is_empty() {
        diagnostics.push("wasm-tools cohort_id must not be empty".to_owned());
    }
    validate_cohort_members(&registry.cohort.members, &mut diagnostics);
    validate_profiles(&registry.profiles, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn cohort_identity(members: &[WasmToolsCohortMember]) -> String {
    let mut sorted = members.to_vec();
    sorted.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(COHORT_IDENTITY_DOMAIN);
    for member in sorted {
        update_hash_field(&mut hasher, member.crate_name.as_bytes());
        update_hash_field(&mut hasher, member.version.as_bytes());
        update_hash_field(&mut hasher, member.role.as_bytes());
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

pub(crate) fn registry_identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

pub(crate) fn analyze_bytes(
    bytes: &[u8],
    registry: &WasmProfileRegistry,
    profile_id: &str,
    registry_identity: String,
    parent_artifacts: &[String],
) -> Result<WasmArtifactEnvelope, Vec<String>> {
    let diagnostics = validate_registry(registry);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let profile = registry
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| vec![format!("unknown Wasm artifact profile `{profile_id}`")])?;
    let exact_artifact_identity = identity(bytes);
    let extraction = extract_facts(bytes, profile);
    let interface_identity = interface_identity(&extraction.facts);
    let mut findings = extraction
        .parse_diagnostics
        .into_iter()
        .map(|diagnostic| WasmArtifactFinding {
            issue_code: diagnostic.issue_code.to_owned(),
            severity: if diagnostic.issue_code == DISALLOWED_PROPOSAL_ISSUE {
                profile.severity.disallowed_kind.clone()
            } else {
                profile.severity.malformed.clone()
            },
            message: diagnostic.message,
            byte_offset: diagnostic.byte_offset,
        })
        .collect::<Vec<_>>();
    findings.extend(evaluate_policy(bytes, &extraction.facts, profile));
    findings.sort_by(|left, right| {
        left.issue_code
            .cmp(&right.issue_code)
            .then(left.byte_offset.cmp(&right.byte_offset))
            .then(left.message.cmp(&right.message))
    });
    let profile_identity = profile_identity(profile)?;
    let sarif = render_sarif(&findings);
    Ok(WasmArtifactEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_owned(),
        details: WasmArtifactReceiptDetails {
            profile_id: profile.profile_id.clone(),
            profile_identity,
            registry_identity,
            cohort_id: registry.cohort.cohort_id.clone(),
            cohort_identity: registry.cohort.identity.clone(),
            exact_artifact_identity,
            interface_identity,
            exact_identity_role: "exact-bytes".to_owned(),
            interface_identity_role: "normalized-interface-non-equivalence".to_owned(),
            input_artifact: INPUT_ARTIFACT_NAME.to_owned(),
            handoff_role: handoff_role(parent_artifacts).to_owned(),
            verification_role: profile.verification_role.clone(),
            facts: extraction.facts,
            findings,
            sarif,
        },
        non_claims: profile.non_claims.clone(),
    })
}

pub(crate) fn render_envelope(envelope: &WasmArtifactEnvelope) -> Result<String, String> {
    serde_json::to_string(envelope).map_err(|error| format!("serialize Wasm artifact envelope: {error}"))
}

pub(crate) fn evaluate_envelope(text: &str) -> WasmEnvelopeEvaluation {
    let envelope: WasmArtifactEnvelope = match serde_json::from_str(text) {
        Ok(envelope) => envelope,
        Err(error) => {
            return WasmEnvelopeEvaluation {
                details: None,
                non_claims: Vec::new(),
                diagnostics: vec![format!("parse Wasm artifact analysis envelope: {error}")],
                status: WasmEvaluationStatus::Failed,
            };
        }
    };
    let mut diagnostics = Vec::new();
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Wasm artifact envelope schema `{}`; expected `{ENVELOPE_SCHEMA_VERSION}`",
            envelope.schema_version
        ));
    }
    validate_receipt_details(&envelope.details, &envelope.non_claims, &mut diagnostics);
    let status = if !diagnostics.is_empty()
        || envelope.details.findings.iter().any(|finding| finding.severity == ERROR_SEVERITY)
    {
        WasmEvaluationStatus::Failed
    } else if envelope.details.findings.is_empty() {
        WasmEvaluationStatus::Passed
    } else {
        WasmEvaluationStatus::ManualReview
    };
    diagnostics.sort();
    diagnostics.dedup();
    WasmEnvelopeEvaluation {
        details: Some(envelope.details),
        non_claims: envelope.non_claims,
        diagnostics,
        status,
    }
}

pub(crate) fn validate_receipt_details(
    details: &WasmArtifactReceiptDetails,
    non_claims: &[String],
    diagnostics: &mut Vec<String>,
) {
    for (field, value) in [
        ("profile_identity", details.profile_identity.as_str()),
        ("registry_identity", details.registry_identity.as_str()),
        ("cohort_identity", details.cohort_identity.as_str()),
        ("exact_artifact_identity", details.exact_artifact_identity.as_str()),
        ("interface_identity", details.interface_identity.as_str()),
    ] {
        if !is_blake3_identity(value) {
            diagnostics.push(format!("Wasm artifact {field} must be a BLAKE3 identity"));
        }
    }
    if details.input_artifact != INPUT_ARTIFACT_NAME {
        diagnostics.push(format!("Wasm artifact input must be `{INPUT_ARTIFACT_NAME}`"));
    }
    if details.exact_identity_role != "exact-bytes" {
        diagnostics.push("Wasm exact identity role must remain exact-bytes".to_owned());
    }
    if details.interface_identity_role != "normalized-interface-non-equivalence" {
        diagnostics.push("Wasm interface identity role must retain the non-equivalence boundary".to_owned());
    }
    if !matches!(details.verification_role.as_str(), RECORDED_ONLY_ROLE | "boundary") {
        diagnostics.push("Wasm artifact verification role must remain recorded_only or boundary".to_owned());
    }
    if !matches!(details.handoff_role.as_str(), STANDALONE_HANDOFF | MANTLE_HANDOFF | EXTERNAL_HANDOFF) {
        diagnostics.push(format!("unsupported Wasm artifact handoff role `{}`", details.handoff_role));
    }
    if !non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("Wasm artifact receipt missing execution non-claim".to_owned());
    }
}

fn extract_facts(bytes: &[u8], profile: &WasmArtifactProfile) -> FactExtraction {
    let mut extraction = FactExtraction::default();
    extraction.facts.artifact_kind = UNKNOWN_KIND.to_owned();
    let mut validator = Validator::new_with_features(validation_features(profile));
    if let Err(error) = validator.validate_all(bytes) {
        let issue_code = if error.missing_wasm_feature().is_some() {
            DISALLOWED_PROPOSAL_ISSUE
        } else {
            MALFORMED_ISSUE
        };
        extraction.parse_diagnostics.push(ParseDiagnostic {
            issue_code,
            message: format!("wasmparser validation failed: {error}"),
            byte_offset: u64::try_from(error.offset()).ok(),
        });
    }
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = match payload {
            Ok(payload) => payload,
            Err(error) => {
                push_parse_error(&mut extraction, "wasmparser parse failed", &error);
                break;
            }
        };
        extract_payload(payload, &mut extraction);
    }
    normalize_facts(&mut extraction.facts);
    extraction
}

fn validation_features(profile: &WasmArtifactProfile) -> WasmFeatures {
    let mut features = WasmFeatures::empty();
    for proposal in &profile.allowed_proposals {
        features |= match proposal.as_str() {
            WASM1_PROPOSAL => WasmFeatures::WASM1,
            WASM2_PROPOSAL => WasmFeatures::WASM2,
            WASM3_PROPOSAL => WasmFeatures::WASM3,
            COMPONENT_MODEL_PROPOSAL => WasmFeatures::COMPONENT_MODEL,
            _ => WasmFeatures::empty(),
        };
    }
    features
}

fn push_parse_error(extraction: &mut FactExtraction, context: &str, error: &wasmparser::BinaryReaderError) {
    extraction.parse_diagnostics.push(ParseDiagnostic {
        issue_code: MALFORMED_ISSUE,
        message: format!("{context}: {error}"),
        byte_offset: u64::try_from(error.offset()).ok(),
    });
}

fn extract_payload(payload: Payload<'_>, extraction: &mut FactExtraction) {
    match payload {
        Payload::Version { encoding, .. } if extraction.facts.artifact_kind == UNKNOWN_KIND => {
            extraction.facts.artifact_kind = match encoding {
                Encoding::Module => MODULE_KIND,
                Encoding::Component => COMPONENT_KIND,
            }
            .to_owned();
        }
        Payload::ImportSection(reader) => {
            for import in reader.into_imports() {
                match import {
                    Ok(import) => extraction.facts.imports.push(WasmInterfaceItem {
                        namespace: import.module.to_owned(),
                        name: import.name.to_owned(),
                        kind: format!("{:?}", import.ty),
                    }),
                    Err(error) => push_parse_error(extraction, "parse core import", &error),
                }
            }
        }
        Payload::ExportSection(reader) => {
            for export in reader {
                match export {
                    Ok(export) => extraction.facts.exports.push(WasmInterfaceItem {
                        namespace: MODULE_KIND.to_owned(),
                        name: export.name.to_owned(),
                        kind: format!("{:?}", export.kind),
                    }),
                    Err(error) => push_parse_error(extraction, "parse core export", &error),
                }
            }
        }
        Payload::ComponentImportSection(reader) => {
            for import in reader {
                match import {
                    Ok(import) => extraction.facts.imports.push(WasmInterfaceItem {
                        namespace: COMPONENT_KIND.to_owned(),
                        name: import.name.name.to_owned(),
                        kind: format!("{:?}", import.ty),
                    }),
                    Err(error) => push_parse_error(extraction, "parse component import", &error),
                }
            }
        }
        Payload::ComponentExportSection(reader) => {
            for export in reader {
                match export {
                    Ok(export) => extraction.facts.exports.push(WasmInterfaceItem {
                        namespace: COMPONENT_KIND.to_owned(),
                        name: export.name.name.to_owned(),
                        kind: format!("{:?}", export.kind),
                    }),
                    Err(error) => push_parse_error(extraction, "parse component export", &error),
                }
            }
        }
        Payload::FunctionSection(reader) => {
            extraction.facts.function_count += u64::from(reader.count());
        }
        Payload::MemorySection(reader) => {
            extraction.facts.memory_count += u64::from(reader.count());
        }
        Payload::TableSection(reader) => {
            extraction.facts.table_count += u64::from(reader.count());
        }
        Payload::StartSection { .. } | Payload::ComponentStartSection { .. } => {
            extraction.facts.has_start = true;
        }
        Payload::CodeSectionEntry(body) => match body.get_operators_reader() {
            Ok(mut operators) => {
                while !operators.eof() {
                    match operators.read() {
                        Ok(Operator::MemoryGrow { .. }) => extraction.facts.memory_grow_count += 1,
                        Ok(_) => {}
                        Err(error) => {
                            push_parse_error(extraction, "parse Wasm operator", &error);
                            break;
                        }
                    }
                }
            }
            Err(error) => push_parse_error(extraction, "read Wasm operators", &error),
        },
        Payload::CustomSection(section) => {
            let name = section.name().to_owned();
            if name == PRODUCERS_SECTION {
                extraction.facts.producer_sections += 1;
            }
            extraction.facts.custom_sections.push(name);
        }
        _ => {}
    }
}

fn normalize_facts(facts: &mut WasmArtifactFacts) {
    facts.imports.sort();
    facts.exports.sort();
    facts.custom_sections.sort();
    facts.worlds = facts
        .imports
        .iter()
        .chain(&facts.exports)
        .filter(|item| item.name.contains(COMPONENT_WORLD_SEPARATOR))
        .map(|item| item.name.clone())
        .collect();
    facts.worlds.sort();
    facts.worlds.dedup();
    facts.proposals = match facts.artifact_kind.as_str() {
        MODULE_KIND => vec![WASM1_PROPOSAL.to_owned()],
        COMPONENT_KIND => vec![COMPONENT_MODEL_PROPOSAL.to_owned()],
        _ => Vec::new(),
    };
}

fn evaluate_policy(bytes: &[u8], facts: &WasmArtifactFacts, profile: &WasmArtifactProfile) -> Vec<WasmArtifactFinding> {
    let mut findings = Vec::new();
    if !profile.allowed_artifact_kinds.iter().any(|kind| kind == &facts.artifact_kind) {
        findings.push(finding(
            DISALLOWED_KIND_ISSUE,
            &profile.severity.disallowed_kind,
            format!("artifact kind `{}` is not allowed by profile `{}`", facts.artifact_kind, profile.profile_id),
        ));
    }
    for proposal in &facts.proposals {
        if !profile.allowed_proposals.iter().any(|allowed| allowed == proposal) {
            findings.push(finding(
                DISALLOWED_PROPOSAL_ISSUE,
                &profile.severity.disallowed_kind,
                format!("proposal `{proposal}` is not allowed by profile `{}`", profile.profile_id),
            ));
        }
    }
    for import in &facts.imports {
        let identity = import_identity(import);
        if !profile.allowed_imports.iter().any(|allowed| allowed == &identity) {
            findings.push(finding(
                DISALLOWED_IMPORT_ISSUE,
                &profile.severity.disallowed_import,
                format!("import `{identity}` is not allowed by profile `{}`", profile.profile_id),
            ));
        }
    }
    if !profile.allow_any_exports && !facts.exports.is_empty() {
        findings.push(finding(
            DISALLOWED_EXPORT_ISSUE,
            &profile.severity.disallowed_import,
            format!("profile `{}` forbids exports", profile.profile_id),
        ));
    }
    if !profile.allowed_worlds.is_empty() {
        for world in &facts.worlds {
            if !profile.allowed_worlds.iter().any(|allowed| allowed == world) {
                findings.push(finding(
                    DISALLOWED_WORLD_ISSUE,
                    &profile.severity.disallowed_world,
                    format!("component world `{world}` is not allowed by profile `{}`", profile.profile_id),
                ));
            }
        }
    }
    let artifact_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    push_limit_finding(
        &mut findings,
        "artifact bytes",
        artifact_bytes,
        profile.resource_limits.max_artifact_bytes,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "imports",
        count(&facts.imports),
        profile.resource_limits.max_imports,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "exports",
        count(&facts.exports),
        profile.resource_limits.max_exports,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "memories",
        facts.memory_count,
        profile.resource_limits.max_memories,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "tables",
        facts.table_count,
        profile.resource_limits.max_tables,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "functions",
        facts.function_count,
        profile.resource_limits.max_functions,
        &profile.severity.resource_limit,
    );
    if !profile.allow_memory_grow && facts.memory_grow_count > 0 {
        findings.push(finding(
            MEMORY_GROWTH_ISSUE,
            &profile.severity.growth,
            format!("profile `{}` forbids memory.grow", profile.profile_id),
        ));
    }
    if !profile.allow_start && facts.has_start {
        findings.push(finding(
            START_FUNCTION_ISSUE,
            &profile.severity.start,
            format!("profile `{}` forbids start functions", profile.profile_id),
        ));
    }
    if profile.producer_metadata == "reject" && facts.producer_sections > 0 {
        findings.push(finding(
            PRODUCER_METADATA_ISSUE,
            &profile.severity.duplicate_metadata,
            format!("profile `{}` forbids producers metadata", profile.profile_id),
        ));
    }
    if profile.duplicate_custom_sections == "reject" {
        for section in duplicate_names(&facts.custom_sections) {
            findings.push(finding(
                DUPLICATE_METADATA_ISSUE,
                &profile.severity.duplicate_metadata,
                format!("duplicate custom section `{section}` is rejected"),
            ));
        }
    }
    findings
}

fn validate_cohort_members(members: &[WasmToolsCohortMember], diagnostics: &mut Vec<String>) {
    if members.is_empty() {
        diagnostics.push("wasm-tools cohort must contain members".to_owned());
        return;
    }
    let mut names = BTreeSet::new();
    for member in members {
        if member.crate_name.trim().is_empty() || member.version.trim().is_empty() || member.role.trim().is_empty() {
            diagnostics.push("wasm-tools cohort members require crate_name, version, and role".to_owned());
        }
        if !names.insert(member.crate_name.as_str()) {
            diagnostics.push(format!("duplicate wasm-tools cohort member `{}`", member.crate_name));
        }
    }
}

fn validate_profiles(profiles: &[WasmArtifactProfile], diagnostics: &mut Vec<String>) {
    if profiles.is_empty() {
        diagnostics.push("Wasm artifact registry must contain profiles".to_owned());
        return;
    }
    let mut ids = BTreeSet::new();
    for profile in profiles {
        if !ids.insert(profile.profile_id.as_str()) {
            diagnostics.push(format!("duplicate Wasm artifact profile `{}`", profile.profile_id));
        }
        if profile.allowed_artifact_kinds.is_empty() || profile.allowed_proposals.is_empty() {
            diagnostics
                .push(format!("Wasm artifact profile `{}` has empty kind or proposal policy", profile.profile_id));
        }
        for proposal in &profile.allowed_proposals {
            if !matches!(proposal.as_str(), WASM1_PROPOSAL | WASM2_PROPOSAL | WASM3_PROPOSAL | COMPONENT_MODEL_PROPOSAL)
            {
                diagnostics.push(format!(
                    "Wasm artifact profile `{}` names unknown proposal `{proposal}`",
                    profile.profile_id
                ));
            }
        }
        if !profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
            diagnostics
                .push(format!("Wasm artifact profile `{}` is missing the execution non-claim", profile.profile_id));
        }
        if !matches!(profile.producer_metadata.as_str(), "recorded-only" | "reject") {
            diagnostics.push(format!(
                "Wasm artifact profile `{}` has unsupported producer metadata posture `{}`",
                profile.profile_id, profile.producer_metadata
            ));
        }
        if !matches!(profile.verification_role.as_str(), RECORDED_ONLY_ROLE | "boundary") {
            diagnostics.push(format!(
                "Wasm artifact profile `{}` requests an unsupported verification role",
                profile.profile_id
            ));
        }
    }
}

fn profile_identity(profile: &WasmArtifactProfile) -> Result<String, Vec<String>> {
    let bytes =
        serde_json::to_vec(profile).map_err(|error| vec![format!("serialize Wasm profile identity: {error}")])?;
    Ok(identity(&bytes))
}

fn interface_identity(facts: &WasmArtifactFacts) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(INTERFACE_IDENTITY_DOMAIN);
    update_hash_field(&mut hasher, facts.artifact_kind.as_bytes());
    for world in &facts.worlds {
        update_hash_field(&mut hasher, world.as_bytes());
    }
    for item in &facts.imports {
        update_interface_item(&mut hasher, b"import", item);
    }
    for item in &facts.exports {
        update_interface_item(&mut hasher, b"export", item);
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

fn update_interface_item(hasher: &mut blake3::Hasher, direction: &[u8], item: &WasmInterfaceItem) {
    update_hash_field(hasher, direction);
    update_hash_field(hasher, item.namespace.as_bytes());
    update_hash_field(hasher, item.name.as_bytes());
    update_hash_field(hasher, item.kind.as_bytes());
}

fn update_hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

fn identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn import_identity(item: &WasmInterfaceItem) -> String {
    format!("{}::{}", item.namespace, item.name)
}

fn push_limit_finding(
    findings: &mut Vec<WasmArtifactFinding>,
    resource: &str,
    observed: u64,
    limit: u64,
    severity: &str,
) {
    if observed > limit {
        findings.push(finding(
            RESOURCE_LIMIT_ISSUE,
            severity,
            format!("{resource} count `{observed}` exceeds profile limit `{limit}`"),
        ));
    }
}

fn count<T>(items: &[T]) -> u64 {
    u64::try_from(items.len()).unwrap_or(u64::MAX)
}

fn duplicate_names(names: &[String]) -> Vec<String> {
    let mut duplicates = Vec::new();
    for window in names.windows(2) {
        if window[0] == window[1] {
            duplicates.push(window[0].clone());
        }
    }
    duplicates.sort();
    duplicates.dedup();
    duplicates
}

fn finding(issue_code: &str, severity: &str, message: String) -> WasmArtifactFinding {
    WasmArtifactFinding {
        issue_code: issue_code.to_owned(),
        severity: severity.to_owned(),
        message,
        byte_offset: None,
    }
}

fn render_sarif(findings: &[WasmArtifactFinding]) -> Value {
    let results = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "ruleId": finding.issue_code,
                "level": finding.severity,
                "message": {"text": finding.message},
                "locations": [{"physicalLocation": {
                    "artifactLocation": {"uri": INPUT_ARTIFACT_NAME},
                    "region": finding.byte_offset.map(|offset| serde_json::json!({"byteOffset": offset})).unwrap_or_else(|| serde_json::json!({}))
                }}]
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {"name": "cargo-octet-wasm-artifact"}},
            "results": results
        }]
    })
}

fn is_blake3_identity(value: &str) -> bool {
    value
        .strip_prefix(BLAKE3_PREFIX)
        .is_some_and(|hex| hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn handoff_role(parent_artifacts: &[String]) -> &'static str {
    if parent_artifacts.is_empty() {
        STANDALONE_HANDOFF
    } else if parent_artifacts.iter().any(|parent| parent.starts_with(MANTLE_PARENT_PREFIX)) {
        MANTLE_HANDOFF
    } else {
        EXTERNAL_HANDOFF
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use wasm_encoder::CustomSection;
    use wasm_encoder::Module;

    use super::*;

    const PROFILE_JSON: &str = include_str!("../../standards/wasm-artifact/generated/profiles.json");
    const PROFILE_ID: &str = "portable-component-baseline";
    const MANTLE_PARENT: &str = "mantle:fixture-materialization";

    #[test]
    fn checked_in_registry_and_cohort_identity_are_fresh() {
        let registry = parse_registry(PROFILE_JSON).expect("profile registry");

        let diagnostics = validate_registry(&registry);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn valid_module_and_component_produce_execution_free_facts() {
        let registry = valid_registry();
        let module = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/positive/module.wat"))
            .expect("module fixture");
        let component = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/positive/component.wat"))
            .expect("component fixture");

        let module_report = analyze(&module, &registry, &[]);
        let component_report = analyze(&component, &registry, &[]);

        assert!(module_report.details.findings.is_empty());
        assert_eq!(module_report.details.facts.artifact_kind, MODULE_KIND);
        assert!(component_report.details.findings.is_empty());
        assert_eq!(component_report.details.facts.artifact_kind, COMPONENT_KIND);
        assert_eq!(module_report.details.verification_role, RECORDED_ONLY_ROLE);
    }

    #[test]
    fn malformed_import_growth_start_and_resource_cases_fail_closed() {
        let registry = valid_registry();
        let function_limit = registry.profiles[0].resource_limits.max_functions;
        let over_function_limit = usize::try_from(function_limit.saturating_add(1)).expect("fixture function count");
        let resource_fixture = format!("(module {})", "(func)".repeat(over_function_limit));
        let cases = [
            (vec![0_u8, 1_u8], MALFORMED_ISSUE),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/sign-extension.wat"))
                    .expect("proposal fixture"),
                DISALLOWED_PROPOSAL_ISSUE,
            ),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/undeclared-wasi.wat"))
                    .expect("import fixture"),
                DISALLOWED_IMPORT_ISSUE,
            ),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/memory-grow.wat"))
                    .expect("growth fixture"),
                MEMORY_GROWTH_ISSUE,
            ),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/start-function.wat"))
                    .expect("start fixture"),
                START_FUNCTION_ISSUE,
            ),
            (wat::parse_str(&resource_fixture).expect("resource fixture"), RESOURCE_LIMIT_ISSUE),
        ];

        for (bytes, expected_issue) in cases {
            let report = analyze(&bytes, &registry, &[]);
            assert!(
                report.details.findings.iter().any(|finding| finding.issue_code == expected_issue),
                "missing {expected_issue}: {:?}",
                report.details.findings
            );
        }
    }

    #[test]
    fn report_bytes_and_finding_order_are_deterministic() {
        let registry = valid_registry();
        let bytes = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/undeclared-wasi.wat"))
            .expect("negative fixture");

        let first = analyze(&bytes, &registry, &[]);
        let second = analyze(&bytes, &registry, &[]);
        let first_json = render_envelope(&first).expect("render first report");
        let second_json = render_envelope(&second).expect("render second report");
        let finding_keys = first
            .details
            .findings
            .iter()
            .map(|finding| (&finding.issue_code, &finding.message, finding.byte_offset))
            .collect::<Vec<_>>();
        let mut sorted_keys = finding_keys.clone();
        sorted_keys.sort();

        assert_eq!(first_json, second_json);
        assert_eq!(finding_keys, sorted_keys);
    }

    #[test]
    fn exact_and_interface_identities_remain_distinct_roles() {
        let registry = valid_registry();
        let first = module_with_custom("metadata", b"first");
        let second = module_with_custom("metadata", b"second");

        let first_report = analyze(&first, &registry, &[]);
        let second_report = analyze(&second, &registry, &[]);

        assert_ne!(first_report.details.exact_artifact_identity, second_report.details.exact_artifact_identity);
        assert_eq!(first_report.details.interface_identity, second_report.details.interface_identity);
        assert_eq!(first_report.details.exact_identity_role, "exact-bytes");
        assert_eq!(first_report.details.interface_identity_role, "normalized-interface-non-equivalence");
    }

    #[test]
    fn duplicate_metadata_and_disallowed_world_policy_are_negative_fixtures() {
        let registry = valid_registry();
        let duplicate = module_with_duplicate_custom("producers");
        let duplicate_report = analyze(&duplicate, &registry, &[]);
        assert!(
            duplicate_report
                .details
                .findings
                .iter()
                .any(|finding| { finding.issue_code == DUPLICATE_METADATA_ISSUE })
        );

        let mut profile = registry.profiles[0].clone();
        profile.allowed_worlds = vec!["example:allowed/world".to_owned()];
        let facts = WasmArtifactFacts {
            artifact_kind: COMPONENT_KIND.to_owned(),
            proposals: vec![COMPONENT_MODEL_PROPOSAL.to_owned()],
            worlds: vec!["example:other/world".to_owned()],
            ..WasmArtifactFacts::default()
        };
        let findings = evaluate_policy(&[], &facts, &profile);
        assert!(findings.iter().any(|finding| finding.issue_code == DISALLOWED_WORLD_ISSUE));

        profile.allow_any_exports = false;
        profile.producer_metadata = "reject".to_owned();
        let restricted_facts = WasmArtifactFacts {
            artifact_kind: MODULE_KIND.to_owned(),
            proposals: vec![WASM1_PROPOSAL.to_owned()],
            exports: vec![WasmInterfaceItem {
                namespace: MODULE_KIND.to_owned(),
                name: "run".to_owned(),
                kind: "Func".to_owned(),
            }],
            producer_sections: 1,
            ..WasmArtifactFacts::default()
        };
        let restricted_findings = evaluate_policy(&[], &restricted_facts, &profile);
        assert!(restricted_findings.iter().any(|finding| finding.issue_code == DISALLOWED_EXPORT_ISSUE));
        assert!(restricted_findings.iter().any(|finding| finding.issue_code == PRODUCER_METADATA_ISSUE));
    }

    #[test]
    fn mantle_handoff_binds_exact_bytes_without_claim_transfer() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");

        let report = analyze(&module, &registry, &[MANTLE_PARENT.to_owned()]);

        assert_eq!(report.details.handoff_role, MANTLE_HANDOFF);
        assert_eq!(report.details.input_artifact, INPUT_ARTIFACT_NAME);
        assert!(report.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM));
    }

    #[test]
    fn receipt_validation_rejects_identity_confusion_and_role_promotion() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");
        let mut report = analyze(&module, &registry, &[]);
        report.details.exact_identity_role = "normalized-interface".to_owned();
        report.details.verification_role = "formal_proof".to_owned();

        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));

        assert_eq!(evaluation.status, WasmEvaluationStatus::Failed);
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("exact-bytes")));
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only or boundary")));
    }

    fn analyze(bytes: &[u8], registry: &WasmProfileRegistry, parents: &[String]) -> WasmArtifactEnvelope {
        analyze_bytes(bytes, registry, PROFILE_ID, identity(PROFILE_JSON.as_bytes()), parents).expect("analysis")
    }

    fn valid_registry() -> WasmProfileRegistry {
        parse_registry(PROFILE_JSON).expect("checked-in profile registry")
    }

    fn module_with_custom(name: &str, data: &[u8]) -> Vec<u8> {
        let mut module = Module::new();
        module.section(&CustomSection {
            name: Cow::Owned(name.to_owned()),
            data: Cow::Owned(data.to_vec()),
        });
        module.finish()
    }

    fn module_with_duplicate_custom(name: &str) -> Vec<u8> {
        let mut module = Module::new();
        let first = CustomSection {
            name: Cow::Owned(name.to_owned()),
            data: Cow::Borrowed(b"first"),
        };
        let second = CustomSection {
            name: Cow::Owned(name.to_owned()),
            data: Cow::Borrowed(b"second"),
        };
        module.section(&first);
        module.section(&second);
        module.finish()
    }
}
