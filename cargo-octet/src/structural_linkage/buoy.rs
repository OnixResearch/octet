//! Bounded intake adapter for pinned `cairn.buoy-comparison.receipt.v1` reports.
//!
//! All upstream Buoy schema assumptions stay confined inside this module. Intake is read-only and
//! rejects any claimed upstream runtime execution or external-authority takeover.

const BUOY_RECEIPT_SCHEMA_V1: &str = "cairn.buoy-comparison.receipt.v1";
const PINNED_BUOY_REVISION: &str = "01096fbdd0b9bf39b2fec468f90e7b1db958f615";
const PINNED_BUOY_HASH_ALGO: &str = "sha256";
const PINNED_BUOY_HASH_MODE: &str = "recursive-nar";

fn default_none<T>() -> Option<T> {
    None
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct UpstreamSummary {
    producer: String,
    repository: String,
    revision: String,
    source_content_hash: String,
    source_content_hash_algorithm: String,
    source_content_hash_mode: String,
    consumer_package: String,
    rust_toolchain: String,
    digest_feature: String,
    commands: Vec<String>,
    #[serde(default = "Vec::new")]
    command_behavior: Vec<String>,
    report_schemas: Vec<String>,
    #[serde(default = "Vec::new")]
    evidence: Vec<String>,
    executed: bool,
    artifact_available: bool,
    disposition: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct MappingSummary {
    id: String,
    semantic_subset: String,
    projection_version: String,
    original_authority: String,
    preserves_original_identities: bool,
    external_projection_is_authority: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DecisionSummary {
    disposition: String,
    runtime_adoption: bool,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ExecutionSummary {
    native_executed: bool,
    upstream_executed: bool,
    case_count: usize,
    executed_case_count: usize,
    declared_only_case_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DeclaredUpstreamObservation {
    disposition: String,
    semantic: String,
    report_schema: String,
    report_blake3: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PilotCase {
    id: String,
    #[serde(default = "default_none")]
    fixture_blake3: Option<String>,
    execution_mode: String,
    #[serde(default = "default_none")]
    base: Option<String>,
    #[serde(default = "default_none")]
    registry_source: Option<String>,
    #[serde(default = "default_none")]
    index_mutation: Option<String>,
    #[serde(default = "default_none")]
    mutation: Option<serde_json::Value>,
    native: serde_json::Value,
    #[serde(default = "default_none")]
    native_semantic: Option<String>,
    declared_upstream: DeclaredUpstreamObservation,
    comparison: String,
    #[serde(default = "default_none")]
    divergence: Option<String>,
    outcome: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ComparisonReceipt {
    schema: String,
    command: String,
    upstream: UpstreamSummary,
    mapping: MappingSummary,
    decision: DecisionSummary,
    manifest_blake3: String,
    policy_blake3: String,
    comparison_identity: String,
    execution: ExecutionSummary,
    cases: Vec<PilotCase>,
    #[serde(default = "default_none")]
    mutation_qualification: Option<serde_json::Value>,
    #[serde(default = "Vec::new")]
    known_disagreements: Vec<serde_json::Value>,
    #[serde(default = "Vec::new")]
    unsupported_cases: Vec<serde_json::Value>,
    #[serde(default = "default_none")]
    producer_identities: Option<serde_json::Value>,
    valid: bool,
    complete: bool,
    verdict: String,
    boundary: String,
    non_claims: Vec<String>,
    issues: Vec<serde_json::Value>,
}

/// Adapts one pinned `cairn.buoy-comparison.receipt.v1` JSON report into bounded records.
pub(crate) fn adapt_report(
    profile: &super::IntakeProfile,
    report_bytes: &[u8],
) -> Result<super::AdapterOutput, String> {
    let raw_schema = serde_json::from_slice::<serde_json::Value>(report_bytes)
        .ok()
        .and_then(|val| val.get("schema").and_then(serde_json::Value::as_str).map(str::to_owned))
        .unwrap_or_default();
    if raw_schema != profile.expected_report_schema || raw_schema != BUOY_RECEIPT_SCHEMA_V1 {
        return Ok(super::AdapterOutput {
            report_schema: raw_schema.clone(),
            records: Vec::new(),
            historical_records: Vec::new(),
            unsupported_reasons: vec![format!(
                "unsupported Buoy receipt schema `{raw_schema}`; expected `{}`",
                profile.expected_report_schema
            )],
            diagnostics: Vec::new(),
        });
    }
    let receipt: ComparisonReceipt = serde_json::from_slice(report_bytes)
        .map_err(|error| format!("parse pinned Buoy comparison receipt: {error}"))?;
    let mut unsupported_reasons = Vec::new();
    let mut diagnostics = Vec::new();
    validate_receipt_invariants(&receipt, &mut unsupported_reasons, &mut diagnostics);
    let records = project_supported_cases(profile, &receipt, &mut unsupported_reasons, &mut diagnostics);
    if records.is_empty() && unsupported_reasons.is_empty() && diagnostics.is_empty() {
        diagnostics.push("pinned Buoy comparison receipt produced zero lossless current cases".to_owned());
    }
    Ok(super::AdapterOutput {
        report_schema: receipt.schema,
        records,
        historical_records: Vec::new(),
        unsupported_reasons,
        diagnostics,
    })
}

fn validate_receipt_invariants(
    receipt: &ComparisonReceipt,
    unsupported: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
) {
    if receipt.upstream.producer != "buoy" || receipt.upstream.revision != PINNED_BUOY_REVISION {
        unsupported.push(format!(
            "unsupported Buoy upstream `{}` at revision `{}`; pinned revision is `{PINNED_BUOY_REVISION}`",
            receipt.upstream.producer, receipt.upstream.revision
        ));
    }
    if receipt.upstream.source_content_hash_algorithm != PINNED_BUOY_HASH_ALGO
        || receipt.upstream.source_content_hash_mode != PINNED_BUOY_HASH_MODE
    {
        diagnostics.push("pinned Buoy source content hash algorithm/mode mismatch".to_owned());
    }
    if receipt.upstream.executed || receipt.execution.upstream_executed {
        diagnostics.push("Buoy pilot receipt must not claim upstream runtime execution".to_owned());
    }
    if !receipt.mapping.preserves_original_identities || receipt.mapping.external_projection_is_authority {
        diagnostics.push("Buoy pilot mapping violates identity preservation or authority limits".to_owned());
    }
    if receipt.decision.runtime_adoption {
        diagnostics.push("Buoy pilot receipt cannot authorize runtime adoption".to_owned());
    }
    if !receipt.valid || !receipt.complete || receipt.verdict != "pass" {
        diagnostics.push(format!("Buoy pilot receipt verdict is non-passing (`{}`)", receipt.verdict));
    }
    if receipt.non_claims.is_empty() {
        diagnostics.push("Buoy pilot receipt is missing non_claims".to_owned());
    }
}

fn project_supported_cases(
    profile: &super::IntakeProfile,
    receipt: &ComparisonReceipt,
    unsupported: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
) -> Vec<super::LinkRecord> {
    let mut records = Vec::with_capacity(receipt.cases.len());
    for case in &receipt.cases {
        let semantic = case.declared_upstream.semantic.as_str();
        if semantic == "unsupported" || semantic == "invalid_output" {
            unsupported.push(format!("Buoy pilot case `{}` carries unsupported semantic `{semantic}`", case.id));
            continue;
        }
        if !matches!(semantic, "current" | "stale_requirement" | "stale_item" | "missing_object" | "ambiguous") {
            unsupported.push(format!("Buoy pilot case `{}` uses unknown semantic `{semantic}`", case.id));
            continue;
        }
        if semantic != "current" || case.outcome != "agreement" || case.native_semantic.as_deref() != Some("current") {
            continue;
        }
        let Some(fixture_hash) = case.fixture_blake3.as_deref().filter(|hash| super::is_blake3_hex(hash)) else {
            diagnostics.push(format!("Buoy pilot current case `{}` is missing a valid fixture_blake3", case.id));
            continue;
        };
        let binding_hash = super::domain_hash_json(
            "octet.buoy-pilot-case-binding.v1",
            &(&receipt.comparison_identity, &case.id, fixture_hash, &case.declared_upstream.report_blake3),
        )
        .unwrap_or_default();
        records.push(super::LinkRecord {
            requirement_id: format!("{}:{}", profile.namespace_prefix, case.id),
            requirement_row_blake3: receipt.manifest_blake3.clone(),
            relation_kind: "verify".to_owned(),
            namespace: profile.namespace_prefix.clone(),
            producer: profile.producer.clone(),
            anchor_kind: profile.expected_anchor_kind.clone(),
            source_path: format!("buoy-pilot:{}", case.id),
            source_file_blake3: fixture_hash.to_owned(),
            marker_span: super::AnchorSpan {
                start_byte: 0,
                end_byte: 0,
                start_line: 0,
                end_line: 0,
            },
            anchor: None,
            configuration: None,
            link_blake3: case.declared_upstream.report_blake3.clone(),
            binding_blake3: binding_hash,
        });
    }
    records
}
