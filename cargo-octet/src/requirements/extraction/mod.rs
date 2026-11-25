//! Effect-extraction requirements profile and cross-claim subject/proof verification (#18).

pub(super) mod adapters;
pub(super) mod optional;

#[cfg(test)]
mod tests;

use serde::Deserialize;
use serde::Serialize;

pub(super) const EXTRACTION_PROFILE_SCHEMA_VERSION: &str = "octet-effect-extraction-profile/v1";

/// Subject reference binding a file locator, BLAKE3 digest, and expected role.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SubjectRef {
    pub(crate) locator: String,
    pub(crate) blake3: String,
    pub(crate) role: String,
}

/// Frozen subjects and configuration inventory for effect-extraction admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExtractionSubjects {
    pub(crate) original_implementation: SubjectRef,
    pub(crate) extracted_core: SubjectRef,
    pub(crate) effect_driver: SubjectRef,
    pub(crate) specification: SubjectRef,
    pub(crate) dependency_closure_blake3: String,
    pub(crate) semantic_profile_id: String,
    pub(crate) target_triple: String,
    pub(crate) toolchain_blake3: String,
}

/// Explicit observational equivalence/refinement contract across original and core+driver.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObservationContract {
    pub(crate) relation: String,
    pub(crate) return_values: String,
    pub(crate) explicit_errors: String,
    pub(crate) operation_order_multiplicity: String,
    pub(crate) externally_visible_state: String,
    pub(crate) destruction_treatment: String,
    pub(crate) panic_unwind_treatment: String,
    pub(crate) allocation_treatment: String,
    pub(crate) termination_treatment: String,
    pub(crate) excluded_concurrency: String,
    pub(crate) input_domain: String,
    pub(crate) preconditions: String,
    pub(crate) causal_environment: String,
}

/// Clause role mapping inside the effect-extraction contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExtractionClauses {
    pub(crate) subjects_clause_id: String,
    pub(crate) core_effects_clause_id: String,
    pub(crate) behavior_preservation_clause_id: String,
    pub(crate) correspondence_clause_id: String,
    pub(crate) freshness_clause_id: String,
    pub(crate) rule_normal_form_clause_id: Option<String>,
    pub(crate) effect_independence_clause_id: Option<String>,
}

/// Optional rule-relative normal form policy (#195).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NormalFormPolicy {
    pub(crate) final_program_blake3: String,
    pub(crate) rule_catalog_blake3: String,
    pub(crate) side_condition_policy: String,
}

/// Optional effect-independence certificate policy (Compiler Lab #211).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EffectIndependencePolicy {
    pub(crate) original_schedule_blake3: String,
    pub(crate) reordered_schedule_blake3: String,
    pub(crate) resource_mapping_blake3: String,
    pub(crate) observer_identity: String,
    pub(crate) relation_kind: String,
}

/// Contract-level effect-extraction profile definition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EffectExtractionProfile {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) pilot_id: String,
    pub(crate) subjects: ExtractionSubjects,
    pub(crate) observation_contract: ObservationContract,
    pub(crate) clauses: ExtractionClauses,
    #[serde(default)]
    pub(crate) approved_assumptions: Vec<String>,
    pub(crate) optional_normal_form: Option<NormalFormPolicy>,
    pub(crate) optional_effect_independence: Option<EffectIndependencePolicy>,
}

/// Emitted summary separating static analysis, formal proof, and accepted assumptions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExtractionSummary {
    pub(crate) profile_id: String,
    pub(crate) strongest_supported_claim: String,
    pub(crate) static_analysis_evidence_ids: Vec<String>,
    pub(crate) formal_proof_evidence_ids: Vec<String>,
    pub(crate) accepted_assumption_ids: Vec<String>,
    pub(crate) remaining_obligations: Vec<String>,
    pub(crate) is_cross_clause_verified: bool,
}

/// Evaluates cross-clause subject invariants and native-evidence bindings for an
/// effect-extraction contract, returning a structured summary and diagnostics.
pub(in crate::requirements) fn evaluate(
    contract: &super::contract::Definition,
    evidence: &super::evidence::Bundle,
    source_root: &std::path::Path,
    clauses: &[super::evaluation::decision::ClauseOutcome],
    diagnostics: &mut Vec<String>,
) -> Option<ExtractionSummary> {
    let profile = contract.effect_extraction.as_ref()?;
    let mut cross_diagnostics = Vec::new();
    validate_profile_header(profile, contract, &mut cross_diagnostics);
    adapters::verify_all_adapters(profile, contract, evidence, source_root, &mut cross_diagnostics);
    let is_cross_clause_verified = cross_diagnostics.is_empty();
    diagnostics.extend(cross_diagnostics.iter().cloned());

    let mut static_ids = Vec::new();
    let mut formal_ids = Vec::new();
    let mut remaining = cross_diagnostics;

    for clause in clauses {
        match clause.disposition.as_str() {
            super::DISPOSITION_FORMAL_PROOF => formal_ids.extend(clause.evidence_ids.iter().cloned()),
            super::DISPOSITION_EMPIRICAL_CHECK => static_ids.extend(clause.evidence_ids.iter().cloned()),
            super::DISPOSITION_UNRESOLVED => {
                remaining.push(format!("unresolved clause `{}`", clause.clause_id));
            }
            _ => {}
        }
        if clause.required
            && !matches!(clause.status.as_str(), super::STATUS_PASSED | super::STATUS_ASSUMPTION_ACCEPTED)
        {
            remaining.push(format!("clause `{}` status `{}`", clause.clause_id, clause.status));
        }
    }
    static_ids.sort();
    static_ids.dedup();
    formal_ids.sort();
    formal_ids.dedup();
    remaining.sort();
    remaining.dedup();

    let accepted_assumption_ids =
        contract.assumptions.iter().map(|assumption| assumption.assumption_id.clone()).collect::<Vec<_>>();
    let strongest_supported_claim = if diagnostics.is_empty() && remaining.is_empty() {
        if accepted_assumption_ids.is_empty() {
            "effect-extraction-verified".to_owned()
        } else {
            "effect-extraction-conditional-on-accepted-assumptions".to_owned()
        }
    } else {
        "effect-extraction-blocked".to_owned()
    };

    Some(ExtractionSummary {
        profile_id: profile.profile_id.clone(),
        strongest_supported_claim,
        static_analysis_evidence_ids: static_ids,
        formal_proof_evidence_ids: formal_ids,
        accepted_assumption_ids,
        remaining_obligations: remaining,
        is_cross_clause_verified,
    })
}

fn validate_profile_header(
    profile: &EffectExtractionProfile,
    contract: &super::contract::Definition,
    diagnostics: &mut Vec<String>,
) {
    if profile.schema_version != EXTRACTION_PROFILE_SCHEMA_VERSION {
        diagnostics.push(format!("unsupported effect-extraction profile schema `{}`", profile.schema_version));
    }
    let required_clauses = [
        profile.clauses.subjects_clause_id.as_str(),
        profile.clauses.core_effects_clause_id.as_str(),
        profile.clauses.behavior_preservation_clause_id.as_str(),
        profile.clauses.correspondence_clause_id.as_str(),
        profile.clauses.freshness_clause_id.as_str(),
    ];
    for clause_id in required_clauses {
        if !contract.clauses.iter().any(|c| c.clause_id == clause_id && c.required) {
            diagnostics.push(format!("effect-extraction required clause `{clause_id}` is missing or optional"));
        }
    }
    if profile.observation_contract.excluded_concurrency.trim().is_empty()
        || profile.observation_contract.operation_order_multiplicity.trim().is_empty()
    {
        diagnostics.push("effect-extraction observation contract is incomplete".to_owned());
    }
}
