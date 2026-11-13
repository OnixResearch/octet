use serde::Deserialize;
use serde::Serialize;

/// Rust-owned contract decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Outcome {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) contract_blake3: String,
    pub(crate) evidence_blake3: String,
    pub(crate) source_snapshot_blake3: String,
    pub(crate) reviewed_specification: super::super::contract::ReviewedSpecification,
    pub(crate) aggregate_result: String,
    pub(crate) clauses: Vec<ClauseOutcome>,
    pub(crate) assumptions: Vec<AssumptionOutcome>,
    pub(crate) exclusions: Vec<super::super::contract::ExcludedBehavior>,
    pub(crate) limitations: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) decision_blake3: String,
}

/// One clause decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClauseOutcome {
    pub(crate) clause_id: String,
    pub(crate) required: bool,
    pub(crate) disposition: String,
    pub(crate) status: String,
    pub(crate) evidence_ids: Vec<String>,
    pub(crate) source_bindings: Vec<super::super::contract::SourceBinding>,
    pub(crate) diagnostics: Vec<String>,
}

/// One assumption decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssumptionOutcome {
    pub(crate) assumption_id: String,
    pub(crate) status: String,
    pub(crate) identity: String,
    pub(crate) trusted_boundary_identity: String,
    pub(crate) supporting_evidence_ids: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
}

pub(in crate::requirements) fn evaluate_contract(
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    contract_bytes: &[u8],
    evidence_bytes: &[u8],
    source_root: &std::path::Path,
) -> Result<Outcome, String> {
    let contract_blake3 = super::super::hash_bytes(contract_bytes);
    let evidence_blake3 = super::super::hash_bytes(evidence_bytes);
    let mut diagnostics = super::super::contract::validation::validate(contract);
    validate_evidence_identity(contract, evidence, &contract_blake3, source_root, &mut diagnostics);
    let source_snapshot_blake3 =
        super::source::snapshot_hash(contract, &evidence.source_snapshot, source_root, &mut diagnostics);
    let assumptions = super::super::evidence::evaluate_assumptions(contract, evidence, &mut diagnostics);
    let mut clauses = Vec::with_capacity(contract.clauses.len());
    let evaluator = super::clause::Context {
        contract,
        evidence,
        source_root,
        assumptions: &assumptions,
    };
    for clause in &contract.clauses {
        clauses.push(super::clause::evaluate(&evaluator, clause, &mut diagnostics));
    }
    if clauses.iter().all(|clause| !clause.required) {
        diagnostics.push("requirement contract has an empty required inventory".to_owned());
    }
    let assumptions_out = assumption_decisions(contract, &assumptions);
    let limitations = contract_limitations(contract);
    if !assumptions_out
        .iter()
        .all(|assumption| assumption.status == super::super::STATUS_ASSUMPTION_ACCEPTED)
    {
        diagnostics.push("one or more accepted assumptions are missing, stale, or unsupported".to_owned());
    }
    let has_required_failure = clauses.iter().any(|clause| {
        clause.required
            && !matches!(
                clause.status.as_str(),
                super::super::STATUS_PASSED | super::super::STATUS_ASSUMPTION_ACCEPTED | super::super::STATUS_EXCLUDED
            )
    });
    let aggregate_result = if diagnostics.is_empty() && !has_required_failure {
        super::super::STATUS_PASSED
    } else {
        super::super::STATUS_FAILED
    }
    .to_owned();
    diagnostics.sort();
    diagnostics.dedup();
    let mut decision = Outcome {
        schema_version: super::super::DECISION_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3,
        evidence_blake3,
        source_snapshot_blake3,
        reviewed_specification: contract.reviewed_specification.clone(),
        aggregate_result,
        clauses,
        assumptions: assumptions_out,
        exclusions: contract.exclusions.clone(),
        limitations,
        diagnostics,
        non_claims: super::super::non_claims(contract),
        decision_blake3: String::new(),
    };
    decision.decision_blake3 = digest(&decision)?;
    Ok(decision)
}

pub(in crate::requirements) fn digest(decision: &Outcome) -> Result<String, String> {
    let mut body = decision.clone();
    body.decision_blake3 = String::new();
    let bytes = serde_json::to_vec(&body).map_err(|error| format!("serialize requirement decision: {error}"))?;
    Ok(super::super::hash_bytes(&bytes))
}

pub(in crate::requirements) fn validate_evidence_identity(
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    contract_blake3: &str,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    if evidence.schema_version != super::super::EVIDENCE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "requirement evidence field `schema_version` expected `{EVIDENCE_SCHEMA_VERSION}` but got `{}`",
            evidence.schema_version,
            EVIDENCE_SCHEMA_VERSION = super::super::EVIDENCE_SCHEMA_VERSION
        ));
    }
    if evidence.contract_id != contract.contract_id {
        diagnostics.push(format!(
            "requirement evidence contract_id `{}` does not match contract `{}`",
            evidence.contract_id, contract.contract_id
        ));
    }
    if let Some(declared) = &evidence.contract_blake3
        && declared != contract_blake3
    {
        diagnostics.push("requirement evidence contract_blake3 does not match the supplied contract bytes".to_owned());
    }
    let spec_path = super::super::resolve_path(source_root, &contract.reviewed_specification.path);
    match std::fs::read(&spec_path) {
        Ok(bytes) => {
            let actual = super::super::hash_bytes(&bytes);
            if actual != contract.reviewed_specification.blake3 {
                diagnostics.push(format!(
                    "reviewed specification `{}` changed: expected {}, actual {actual}",
                    contract.reviewed_specification.path, contract.reviewed_specification.blake3
                ));
            }
        }
        Err(error) => diagnostics
            .push(format!("reviewed specification `{}` is missing: {error}", contract.reviewed_specification.path)),
    }
}

pub(in crate::requirements) fn contract_limitations(contract: &super::super::contract::Definition) -> Vec<String> {
    let mut limitations = Vec::with_capacity(contract.assumptions.len() + contract.exclusions.len());
    for assumption in &contract.assumptions {
        limitations.push(format!(
            "accepted assumption `{}` is bound to identity {} and trust boundary {}",
            assumption.assumption_id, assumption.identity, assumption.trusted_boundary_identity
        ));
    }
    for exclusion in &contract.exclusions {
        limitations.push(format!(
            "excluded behavior `{}` for clause `{}`: {}",
            exclusion.exclusion_id, exclusion.clause_id, exclusion.reason
        ));
    }
    limitations
}

fn assumption_decisions(
    contract: &super::super::contract::Definition,
    assumptions: &std::collections::BTreeMap<String, AssumptionOutcome>,
) -> Vec<AssumptionOutcome> {
    let mut assumptions_out = Vec::with_capacity(contract.assumptions.len());
    for assumption in &contract.assumptions {
        let decision = assumptions.get(&assumption.assumption_id).cloned().unwrap_or_else(|| AssumptionOutcome {
            assumption_id: assumption.assumption_id.clone(),
            status: super::super::STATUS_MISSING.to_owned(),
            identity: assumption.identity.clone(),
            trusted_boundary_identity: assumption.trusted_boundary_identity.clone(),
            supporting_evidence_ids: Vec::new(),
            diagnostics: vec![format!(
                "assumption `{}` has no evidence record",
                assumption.assumption_id
            )],
        });
        assumptions_out.push(decision);
    }
    assumptions_out
}
