//! Bounded intake and read-only verification for Cairn structural trace receipts and pinned Buoy
//! comparison pilot reports.

pub(crate) mod buoy;
pub(crate) mod cairn;
pub(crate) mod receipt;
pub(crate) mod source;

#[cfg(test)]
mod fixture_gen;
#[cfg(test)]
mod tests;

pub(crate) const PROFILES_SCHEMA_V1: &str = "octet-structural-linkage-profiles/v1";
pub(crate) const PROJECTION_SCHEMA_V1: &str = "octet-structural-linkage-projection/v1";
pub(crate) const RECEIPT_SCHEMA_V1: &str = "octet-structural-linkage-receipt/v1";
pub(crate) const ANCHOR_KIND_RUST_ITEM_V1: &str = "rust_item_v1";
pub(crate) const EVIDENCE_CLASS_NAME: &str = "structural_linkage";
pub(crate) const EVIDENCE_TIER_NAME: &str = "structural_linkage";
pub(crate) const PROJECTION_DOMAIN: &str = "octet.structural-linkage.projection.v1";
pub(crate) const RECEIPT_DOMAIN: &str = "octet.structural-linkage.receipt.v1";
pub(crate) const MAX_INTAKE_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const STATUS_VALID: &str = "valid";
pub(crate) const STATUS_STALE_BINDING: &str = "stale_binding";
pub(crate) const STATUS_UNSUPPORTED: &str = "unsupported";
pub(crate) const STATUS_FAILED: &str = "failed";
pub(crate) const SOURCE_STATUS_CURRENT: &str = "current";
pub(crate) const SOURCE_STATUS_FORMATTING_ONLY: &str = "formatting_only";
pub(crate) const SOURCE_STATUS_BODY_CHANGED: &str = "item_body_changed";
pub(crate) const SOURCE_STATUS_MISSING: &str = "missing";
pub(crate) const SOURCE_STATUS_UNREADABLE: &str = "unreadable";

const BLAKE3_HEX_LENGTH: usize = 64;

/// Reviewed Nickel-authored intake profile registry for trace-anchor reports.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntakeProfileRegistry {
    pub(crate) schema_version: String,
    pub(crate) profiles: Vec<IntakeProfile>,
}

/// One reviewed producer intake profile.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntakeProfile {
    pub(crate) profile_id: String,
    pub(crate) producer: String,
    pub(crate) expected_report_schema: String,
    pub(crate) expected_anchor_kind: String,
    pub(crate) expected_adapter: String,
    pub(crate) expected_parser: String,
    pub(crate) expected_structural_schema: String,
    pub(crate) mapping_id: String,
    pub(crate) mapping_version: String,
    pub(crate) namespace_prefix: String,
    pub(crate) evidence_class: String,
    pub(crate) evidence_tier: String,
    pub(crate) supported_relations: Vec<String>,
    pub(crate) command_execution_allowed: bool,
    pub(crate) non_claims: Vec<String>,
}

/// Parser interpretation settings recorded alongside one binding.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ItemConfiguration {
    pub(crate) source_path: String,
    pub(crate) file_blake3: String,
    pub(crate) schema_version: String,
    pub(crate) target: String,
    pub(crate) features: Vec<String>,
    pub(crate) declared_features: Vec<String>,
    pub(crate) cfg: Vec<String>,
}

/// Byte/line span of a source marker.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AnchorSpan {
    pub(crate) start_byte: u64,
    pub(crate) end_byte: u64,
    pub(crate) start_line: u64,
    pub(crate) end_line: u64,
}

/// Parser-resolved Rust item anchor.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ParsedAnchor {
    pub(crate) source_path: String,
    pub(crate) marker_start_byte: u64,
    pub(crate) start_byte: u64,
    pub(crate) end_byte: u64,
    pub(crate) item_kind: String,
    pub(crate) scoped_locator: String,
    pub(crate) normalized_item_blake3: String,
}

/// One active projected requirement-to-source record.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkRecord {
    pub(crate) requirement_id: String,
    pub(crate) requirement_row_blake3: String,
    pub(crate) relation_kind: String,
    pub(crate) namespace: String,
    pub(crate) producer: String,
    pub(crate) anchor_kind: String,
    pub(crate) source_path: String,
    pub(crate) source_file_blake3: String,
    pub(crate) marker_span: AnchorSpan,
    pub(crate) anchor: Option<ParsedAnchor>,
    pub(crate) configuration: Option<ItemConfiguration>,
    pub(crate) link_blake3: String,
    pub(crate) binding_blake3: String,
}

/// Historical binding retained solely for auditability; never admitted as current evidence.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoricalRecord {
    pub(crate) requirement_id: String,
    pub(crate) source_path: String,
    pub(crate) binding_blake3: String,
    pub(crate) historical_only: bool,
    pub(crate) note: String,
}

/// Octet-converted projection preserving original producer report identity and namespace.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_field_names,
    reason = "JSON wire schema exposes explicit `projection_blake3` identity field"
)]
pub(crate) struct Projection {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) producer: String,
    pub(crate) original_report_schema: String,
    pub(crate) original_report_blake3: String,
    pub(crate) mapping_id: String,
    pub(crate) mapping_version: String,
    pub(crate) namespace: String,
    pub(crate) evidence_class: String,
    pub(crate) evidence_tier: String,
    pub(crate) records: Vec<LinkRecord>,
    pub(crate) historical_records: Vec<HistoricalRecord>,
    pub(crate) unsupported_reasons: Vec<String>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) projection_blake3: String,
}

/// Source-recheck result distinguishing formatting edits from item-body edits.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceMatchOutcome {
    pub(crate) locator: String,
    pub(crate) expected_file_blake3: String,
    pub(crate) actual_file_blake3: String,
    pub(crate) expected_normalized_item_blake3: String,
    pub(crate) actual_normalized_item_blake3: String,
    pub(crate) status: String,
    pub(crate) explanation: String,
}

/// Receipt binding original report, converted projection, source checks, and non-claims.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversionReceipt {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) producer: String,
    pub(crate) original_report_schema: String,
    pub(crate) original_report_blake3: String,
    pub(crate) projection_schema_version: String,
    pub(crate) projection_blake3: String,
    pub(crate) mapping_id: String,
    pub(crate) mapping_version: String,
    pub(crate) namespace: String,
    pub(crate) evidence_class: String,
    pub(crate) evidence_tier: String,
    pub(crate) status: String,
    pub(crate) command_executed: bool,
    pub(crate) replay_deterministic: bool,
    pub(crate) active_record_count: usize,
    pub(crate) historical_record_count: usize,
    pub(crate) source_checks: Vec<SourceMatchOutcome>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) receipt_blake3: String,
}

/// Intermediate result returned by a producer adapter before projection and receipt hashing.
pub(crate) struct AdapterOutput {
    pub(crate) report_schema: String,
    pub(crate) records: Vec<LinkRecord>,
    pub(crate) historical_records: Vec<HistoricalRecord>,
    pub(crate) unsupported_reasons: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
}

/// Parses and validates an exported `octet-structural-linkage-profiles/v1` registry.
pub(crate) fn parse_profile_registry(bytes: &[u8]) -> Result<IntakeProfileRegistry, String> {
    let registry: IntakeProfileRegistry =
        serde_json::from_slice(bytes).map_err(|error| format!("parse structural intake profile registry: {error}"))?;
    if registry.schema_version != PROFILES_SCHEMA_V1 {
        return Err(format!("unsupported structural intake profile registry schema `{}`", registry.schema_version));
    }
    if registry.profiles.is_empty() {
        return Err("structural intake profile registry has no profiles".to_owned());
    }
    for profile in &registry.profiles {
        validate_profile(profile)?;
    }
    Ok(registry)
}

/// Selects one validated intake profile by `profile_id`.
pub(crate) fn select_profile<'a>(
    registry: &'a IntakeProfileRegistry,
    profile_id: &str,
) -> Result<&'a IntakeProfile, String> {
    registry
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown structural intake profile `{profile_id}`"))
}

/// Checks whether a string is a 64-character lowercase BLAKE3 hex digest.
pub(crate) fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// Computes raw BLAKE3 lowercase hex digest over bytes.
pub(crate) fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Computes Cairn/Octet domain-separated BLAKE3 digest (`domain \0 bytes`).
pub(crate) fn domain_hash_bytes(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

/// Computes domain-separated hash over canonical JSON representation of `value`.
pub(crate) fn domain_hash_json<T: serde::Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let json_val = serde_json::to_value(value).map_err(|error| format!("serialize hash input: {error}"))?;
    Ok(domain_hash_bytes(domain, receipt::canonical_json(&json_val).as_bytes()))
}

fn validate_profile(profile: &IntakeProfile) -> Result<(), String> {
    if profile.expected_anchor_kind != ANCHOR_KIND_RUST_ITEM_V1 {
        return Err(format!("unsupported anchor kind `{}` in `{}`", profile.expected_anchor_kind, profile.profile_id));
    }
    if profile.evidence_class != EVIDENCE_CLASS_NAME || profile.evidence_tier != EVIDENCE_TIER_NAME {
        return Err(format!("profile `{}` must stay scoped to `{EVIDENCE_CLASS_NAME}`", profile.profile_id));
    }
    if profile.command_execution_allowed {
        return Err(format!("profile `{}` must forbid runtime execution", profile.profile_id));
    }
    if profile.non_claims.is_empty() || profile.supported_relations.is_empty() {
        return Err(format!("profile `{}` requires non-empty relations and non_claims", profile.profile_id));
    }
    Ok(())
}

/// Reads a bounded report/projection/receipt bundle from disk and verifies clause matching.
pub(crate) fn verify_clause_paths(
    paths: [&std::path::Path; 4],
    expected_profile: &str,
    artifact_root: &std::path::Path,
    clause_ids: (&str, &[String]),
    checked_scope: &[String],
) -> Result<ConversionReceipt, (String, Vec<String>)> {
    let mut buffers = Vec::with_capacity(paths.len());
    for path in paths {
        let meta = std::fs::metadata(path)
            .map_err(|err| (STATUS_FAILED.to_owned(), vec![format!("unreadable `{}`: {err}", path.display())]))?;
        if meta.len() > MAX_INTAKE_BYTES {
            return Err((STATUS_FAILED.to_owned(), vec![format!("`{}` exceeds byte cap", path.display())]));
        }
        let bytes = std::fs::read(path)
            .map_err(|err| (STATUS_FAILED.to_owned(), vec![format!("unreadable `{}`: {err}", path.display())]))?;
        buffers.push(bytes);
    }
    receipt::verify_clause_files(
        [&buffers[0], &buffers[1], &buffers[2], &buffers[3]],
        expected_profile,
        artifact_root,
        clause_ids,
        checked_scope,
    )
}
