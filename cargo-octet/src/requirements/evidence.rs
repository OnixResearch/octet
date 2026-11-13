use serde::Deserialize;
use serde::Serialize;

/// Evidence bundle consumed by contract evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Bundle {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) contract_blake3: Option<String>,
    #[serde(default = "Vec::new")]
    pub(crate) source_snapshot: Vec<super::contract::SourceBinding>,
    #[serde(default = "Vec::new")]
    pub(crate) proof_decks: Vec<ProofDeck>,
    #[serde(default = "Vec::new")]
    pub(crate) empirical_checks: Vec<EmpiricalCheck>,
    #[serde(default = "Vec::new")]
    pub(crate) assumption_records: Vec<AssumptionRecord>,
    #[serde(default = "Vec::new")]
    pub(crate) non_claims: Vec<String>,
}

/// One retained proof-card deck.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProofDeck {
    pub(crate) evidence_id: String,
    pub(crate) deck: String,
    #[serde(default = "super::default_dot")]
    pub(crate) artifact_root: String,
}

/// One empirical check record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct EmpiricalCheck {
    pub(crate) evidence_id: String,
    pub(crate) evidence_class: String,
    pub(crate) status: String,
    pub(crate) checker_id: String,
    #[serde(default = "Vec::new")]
    pub(crate) command_argv: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) checked_scope: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) source_bindings: Vec<super::contract::SourceBinding>,
    pub(crate) artifact: Option<String>,
    pub(crate) artifact_blake3: Option<String>,
    #[serde(default = "Vec::new")]
    pub(crate) diagnostics: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) non_claims: Vec<String>,
}

/// One assumption evidence record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssumptionRecord {
    pub(crate) assumption_id: String,
    pub(crate) identity: String,
    pub(crate) trusted_boundary_identity: String,
    pub(crate) status: String,
    #[serde(default = "Vec::new")]
    pub(crate) supporting_evidence_ids: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) diagnostics: Vec<String>,
}

pub(super) fn evaluate_assumptions(
    contract: &super::contract::Definition,
    evidence: &Bundle,
    diagnostics: &mut Vec<String>,
) -> std::collections::BTreeMap<String, super::evaluation::decision::AssumptionOutcome> {
    let empirical = evidence
        .empirical_checks
        .iter()
        .map(|check| (check.evidence_id.as_str(), check))
        .collect::<std::collections::BTreeMap<_, _>>();
    contract
        .assumptions
        .iter()
        .map(|assumption| {
            let record =
                evidence.assumption_records.iter().find(|record| record.assumption_id == assumption.assumption_id);
            let mut assumption_diagnostics = Vec::new();
            let Some(record) = record else {
                assumption_diagnostics
                    .push(format!("assumption `{}` has no retained evidence record", assumption.assumption_id));
                return (assumption.assumption_id.clone(), super::evaluation::decision::AssumptionOutcome {
                    assumption_id: assumption.assumption_id.clone(),
                    status: super::STATUS_MISSING.to_owned(),
                    identity: assumption.identity.clone(),
                    trusted_boundary_identity: assumption.trusted_boundary_identity.clone(),
                    supporting_evidence_ids: Vec::new(),
                    diagnostics: assumption_diagnostics,
                });
            };
            let supporting = validate_assumption_record(assumption, record, &empirical, &mut assumption_diagnostics);
            assumption_diagnostics.sort();
            assumption_diagnostics.dedup();
            diagnostics.extend(
                assumption_diagnostics
                    .iter()
                    .map(|diagnostic| format!("assumption `{}`: {diagnostic}", assumption.assumption_id)),
            );
            let status = if assumption_diagnostics.is_empty() {
                super::STATUS_ASSUMPTION_ACCEPTED
            } else {
                super::STATUS_FAILED
            };
            if status != super::STATUS_ASSUMPTION_ACCEPTED {
                diagnostics.push(format!("assumption `{}` is not accepted", assumption.assumption_id));
            }
            (assumption.assumption_id.clone(), super::evaluation::decision::AssumptionOutcome {
                assumption_id: assumption.assumption_id.clone(),
                status: status.to_owned(),
                identity: record.identity.clone(),
                trusted_boundary_identity: record.trusted_boundary_identity.clone(),
                supporting_evidence_ids: supporting,
                diagnostics: assumption_diagnostics,
            })
        })
        .collect()
}

pub(super) fn validate_assumption_record(
    assumption: &super::contract::AcceptedAssumption,
    record: &AssumptionRecord,
    empirical: &std::collections::BTreeMap<&str, &EmpiricalCheck>,
    diagnostics: &mut Vec<String>,
) -> Vec<String> {
    if record.status != super::ASSUMPTION_STATUS_ACCEPTED {
        diagnostics.push(format!("assumption `{}` evidence status is `{}`", assumption.assumption_id, record.status));
    }
    if record.identity != assumption.identity {
        diagnostics.push(format!(
            "assumption `{}` identity changed: expected {}, retained {}",
            assumption.assumption_id, assumption.identity, record.identity
        ));
    }
    if record.trusted_boundary_identity != assumption.trusted_boundary_identity {
        diagnostics.push(format!(
            "assumption `{}` trusted boundary changed: expected {}, retained {}",
            assumption.assumption_id, assumption.trusted_boundary_identity, record.trusted_boundary_identity
        ));
    }
    let supporting = assumption
        .required_empirical_evidence
        .iter()
        .filter_map(|required_id| {
            let Some(check) = empirical.get(required_id.as_str()) else {
                diagnostics.push(format!(
                    "assumption `{}` requires missing empirical evidence `{required_id}`",
                    assumption.assumption_id
                ));
                return None;
            };
            if check.status != super::CHECK_STATUS_PASSED {
                diagnostics.push(format!(
                    "assumption `{}` required empirical evidence `{required_id}` status is `{}`",
                    assumption.assumption_id, check.status
                ));
            }
            Some(required_id.clone())
        })
        .collect::<Vec<_>>();
    for supplied in &record.supporting_evidence_ids {
        if !supporting.contains(supplied) {
            diagnostics.push(format!(
                "assumption `{}` evidence record cites unsupported evidence `{supplied}`",
                assumption.assumption_id
            ));
        }
    }
    for diagnostic in &record.diagnostics {
        diagnostics.push(format!("assumption `{}`: {diagnostic}", assumption.assumption_id));
    }
    supporting
}
