use serde::Serialize;

/// Verification report for a stored decision.
#[derive(Clone, Debug, Serialize)]
pub(in crate::requirements) struct ValidationReport {
    pub(in crate::requirements) schema_version: String,
    pub(in crate::requirements) status: String,
    pub(in crate::requirements) contract_id: String,
    pub(in crate::requirements) recorded_decision_blake3: String,
    pub(in crate::requirements) recomputed_decision_blake3: String,
    pub(in crate::requirements) aggregate_result: String,
    pub(in crate::requirements) diagnostics: Vec<String>,
    pub(in crate::requirements) non_claims: Vec<String>,
}

pub(in crate::requirements) fn compare_decisions(
    recorded: &super::decision::Outcome,
    recomputed: &super::decision::Outcome,
    diagnostics: &mut Vec<String>,
) {
    if recorded.decision_blake3 != recomputed.decision_blake3 {
        diagnostics.push(format!(
            "recorded decision identity {} does not match the recomputed identity {}",
            recorded.decision_blake3, recomputed.decision_blake3
        ));
    }
    for (field, recorded_value, recomputed_value) in [
        ("contract_blake3", recorded.contract_blake3.as_str(), recomputed.contract_blake3.as_str()),
        ("evidence_blake3", recorded.evidence_blake3.as_str(), recomputed.evidence_blake3.as_str()),
        (
            "source_snapshot_blake3",
            recorded.source_snapshot_blake3.as_str(),
            recomputed.source_snapshot_blake3.as_str(),
        ),
        ("aggregate_result", recorded.aggregate_result.as_str(), recomputed.aggregate_result.as_str()),
    ] {
        if recorded_value != recomputed_value {
            diagnostics.push(format!(
                "recorded decision {field} `{recorded_value}` does not match recomputed `{recomputed_value}`"
            ));
        }
    }
    if recorded.clauses != recomputed.clauses {
        diagnostics.push("recorded clause decisions do not match the recomputed decisions".to_owned());
    }
    if recorded.assumptions != recomputed.assumptions {
        diagnostics.push("recorded assumption decisions do not match the recomputed decisions".to_owned());
    }
    if recorded.exclusions != recomputed.exclusions {
        diagnostics.push("recorded exclusions do not match the recomputed exclusions".to_owned());
    }
    if recorded.reviewed_specification != recomputed.reviewed_specification {
        diagnostics.push("recorded reviewed specification identity does not match the contract".to_owned());
    }
    let declared = super::decision::digest(recorded).unwrap_or_default();
    if declared != recorded.decision_blake3 {
        diagnostics.push("recorded decision bytes are stale or tampered".to_owned());
    }
}
