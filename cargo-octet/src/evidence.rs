use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub const PROVENANCE_SCHEMA_VERSION: &str = "octet-provenance-record/v1";
const FIELD_SCHEMA_VERSION: &str = "schema_version";
const FIELD_SOURCE_ID: &str = "source_id";
const FIELD_ARTIFACT_LOCATOR: &str = "artifact_locator";
const FIELD_VERIFIER_IDENTITY: &str = "verifier_identity";
const FIELD_PROOF_ARTIFACT_IDENTITY: &str = "proof_artifact_identity";
const FIELD_HUMAN_ACTION: &str = "human_action";
const UNKNOWN_CHECKER_PREFIX: &str = "unknown checker_id";
const HASH_MISMATCH_PREFIX: &str = "hash mismatch for artifact locator";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    StaticLint,
    PolicyCheck,
    UiFixture,
    AutofixVerified,
    TranscriptAssertion,
    FormalProof,
    ManualReviewRequired,
}

impl EvidenceTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaticLint => "static_lint",
            Self::PolicyCheck => "policy_check",
            Self::UiFixture => "ui_fixture",
            Self::AutofixVerified => "autofix_verified",
            Self::TranscriptAssertion => "transcript_assertion",
            Self::FormalProof => "formal_proof",
            Self::ManualReviewRequired => "manual_review_required",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProvenanceRecord {
    pub schema_version: String,
    pub source_id: String,
    pub checker_id: String,
    pub evidence_tier: EvidenceTier,
    pub artifact_locator: String,
    pub blake3: String,
    pub checked_scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_artifact_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_action: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_artifacts: Vec<String>,
}

pub struct ProvenanceValidationInput<'a> {
    pub known_checker_ids: &'a BTreeSet<String>,
    pub expected_artifact_hashes: &'a BTreeMap<String, String>,
}

pub fn artifact_blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn validate_provenance_record(record: &ProvenanceRecord, input: &ProvenanceValidationInput<'_>) -> Vec<String> {
    let mut diagnostics = Vec::new();

    if is_blank(&record.schema_version) {
        diagnostics.push(missing_field(FIELD_SCHEMA_VERSION));
    } else if record.schema_version != PROVENANCE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported provenance schema_version `{}`; expected `{}`",
            record.schema_version, PROVENANCE_SCHEMA_VERSION
        ));
    }

    if is_blank(&record.source_id) {
        diagnostics.push(missing_field(FIELD_SOURCE_ID));
    }

    if !input.known_checker_ids.contains(&record.checker_id) {
        diagnostics.push(format!("{UNKNOWN_CHECKER_PREFIX} `{}`", record.checker_id));
    }

    if is_blank(&record.artifact_locator) {
        diagnostics.push(missing_field(FIELD_ARTIFACT_LOCATOR));
    } else {
        match input.expected_artifact_hashes.get(&record.artifact_locator) {
            Some(expected_hash) if expected_hash == &record.blake3 => {}
            Some(expected_hash) => diagnostics.push(format!(
                "{HASH_MISMATCH_PREFIX} `{}`; expected blake3 {}, got {}",
                record.artifact_locator, expected_hash, record.blake3
            )),
            None => diagnostics.push(format!(
                "{HASH_MISMATCH_PREFIX} `{}`; expected artifact was not provided",
                record.artifact_locator
            )),
        }
    }

    if record.evidence_tier == EvidenceTier::FormalProof {
        if option_is_blank(record.verifier_identity.as_deref()) {
            diagnostics.push(missing_field(FIELD_VERIFIER_IDENTITY));
        }
        if option_is_blank(record.proof_artifact_identity.as_deref()) {
            diagnostics.push(missing_field(FIELD_PROOF_ARTIFACT_IDENTITY));
        }
    }

    if record.evidence_tier == EvidenceTier::ManualReviewRequired && option_is_blank(record.human_action.as_deref()) {
        diagnostics.push(missing_field(FIELD_HUMAN_ACTION));
    }

    diagnostics
}

fn missing_field(field: &str) -> String {
    format!("missing required provenance field `{field}`")
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn option_is_blank(value: Option<&str>) -> bool {
    value.is_none_or(is_blank)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN_CHECKER_ID: &str = "octet-standards";
    const SOURCE_ID: &str = "r[octet.evidence_provenance.record]";
    const ARTIFACT_LOCATOR: &str = "artifacts/octet/status.json";
    const ARTIFACT_BYTES: &[u8] = b"{\"status\":\"clean\"}\n";
    const HUMAN_ACTION: &str = "review the named suppression before release";
    const PROOF_VERIFIER: &str = "verus@2026-06-30";
    const PROOF_ARTIFACT: &str = "proofs/bounds.verus.receipt";

    #[test]
    fn all_evidence_tiers_round_trip_with_stable_discriminants() {
        let cases = [
            (EvidenceTier::StaticLint, "static_lint"),
            (EvidenceTier::PolicyCheck, "policy_check"),
            (EvidenceTier::UiFixture, "ui_fixture"),
            (EvidenceTier::AutofixVerified, "autofix_verified"),
            (EvidenceTier::TranscriptAssertion, "transcript_assertion"),
            (EvidenceTier::FormalProof, "formal_proof"),
            (EvidenceTier::ManualReviewRequired, "manual_review_required"),
        ];

        for (tier, discriminant) in cases {
            assert_eq!(tier.as_str(), discriminant);
            let rendered = serde_json::to_string(&tier).expect("evidence tier should serialize");
            assert_eq!(rendered, format!("\"{discriminant}\""));
            let parsed: EvidenceTier = serde_json::from_str(&rendered).expect("evidence tier should deserialize");
            assert_eq!(parsed, tier);
        }
    }

    #[test]
    fn unknown_evidence_tier_rejects() {
        let parsed = serde_json::from_str::<EvidenceTier>("\"runtime_magic\"");

        assert!(parsed.is_err());
    }

    #[test]
    fn valid_static_lint_provenance_record_passes() {
        let record = valid_record(EvidenceTier::StaticLint);

        let diagnostics = validate_with_defaults(&record);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn formal_proof_provenance_requires_proof_identity() {
        let record = valid_record(EvidenceTier::FormalProof);

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec![
            "missing required provenance field `verifier_identity`".to_owned(),
            "missing required provenance field `proof_artifact_identity`".to_owned(),
        ]);
    }

    #[test]
    fn formal_proof_provenance_accepts_scoped_proof_identity() {
        let record = ProvenanceRecord {
            verifier_identity: Some(PROOF_VERIFIER.to_owned()),
            proof_artifact_identity: Some(PROOF_ARTIFACT.to_owned()),
            ..valid_record(EvidenceTier::FormalProof)
        };

        let diagnostics = validate_with_defaults(&record);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn manual_review_required_provenance_requires_human_action() {
        let record = valid_record(EvidenceTier::ManualReviewRequired);

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec!["missing required provenance field `human_action`".to_owned()]);
    }

    #[test]
    fn manual_review_required_provenance_accepts_human_action() {
        let record = ProvenanceRecord {
            human_action: Some(HUMAN_ACTION.to_owned()),
            ..valid_record(EvidenceTier::ManualReviewRequired)
        };

        let diagnostics = validate_with_defaults(&record);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn provenance_validation_rejects_missing_source_id() {
        let record = ProvenanceRecord {
            source_id: String::new(),
            ..valid_record(EvidenceTier::PolicyCheck)
        };

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec!["missing required provenance field `source_id`".to_owned()]);
    }

    #[test]
    fn provenance_validation_rejects_unknown_checker_id() {
        let record = ProvenanceRecord {
            checker_id: "unknown-checker".to_owned(),
            ..valid_record(EvidenceTier::PolicyCheck)
        };

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec!["unknown checker_id `unknown-checker`".to_owned()]);
    }

    #[test]
    fn provenance_validation_rejects_hash_mismatch() {
        let record = ProvenanceRecord {
            blake3: artifact_blake3_hex(b"different artifact bytes"),
            ..valid_record(EvidenceTier::TranscriptAssertion)
        };

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec![format!(
            "hash mismatch for artifact locator `{ARTIFACT_LOCATOR}`; expected blake3 {}, got {}",
            artifact_blake3_hex(ARTIFACT_BYTES),
            artifact_blake3_hex(b"different artifact bytes")
        )]);
    }

    fn valid_record(evidence_tier: EvidenceTier) -> ProvenanceRecord {
        ProvenanceRecord {
            schema_version: PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: SOURCE_ID.to_owned(),
            checker_id: KNOWN_CHECKER_ID.to_owned(),
            evidence_tier,
            artifact_locator: ARTIFACT_LOCATOR.to_owned(),
            blake3: artifact_blake3_hex(ARTIFACT_BYTES),
            checked_scope: "cargo-octet sidecar fixture".to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: Vec::new(),
        }
    }

    fn validate_with_defaults(record: &ProvenanceRecord) -> Vec<String> {
        let mut known_checker_ids = BTreeSet::new();
        known_checker_ids.insert(KNOWN_CHECKER_ID.to_owned());

        let mut expected_artifact_hashes = BTreeMap::new();
        expected_artifact_hashes.insert(ARTIFACT_LOCATOR.to_owned(), artifact_blake3_hex(ARTIFACT_BYTES));

        validate_provenance_record(record, &ProvenanceValidationInput {
            known_checker_ids: &known_checker_ids,
            expected_artifact_hashes: &expected_artifact_hashes,
        })
    }
}
