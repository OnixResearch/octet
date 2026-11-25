//! Optional rule-relative normal form (#195) and effect-independence (#211) adapters.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NormalFormReceipt {
    schema_version: String,
    status: String,
    final_program_blake3: String,
    rule_catalog_blake3: String,
    side_condition_policy: String,
    contexts: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EffectIndependenceReceipt {
    schema_version: String,
    status: String,
    extracted_core_blake3: String,
    driver_blake3: String,
    original_schedule_blake3: String,
    reordered_schedule_blake3: String,
    resource_mapping_blake3: String,
    observer_identity: String,
    relation_kind: String,
    is_footprint_soundness_proved: bool,
    is_interchange_theorem_passed: bool,
}

pub(super) fn verify_normal_form_adapter(
    profile: &super::EffectExtractionProfile,
    clause_id: &str,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let Some(policy) = &profile.optional_normal_form else {
        diagnostics.push("optional rule-relative normal form clause requested without policy".to_owned());
        return;
    };
    let path = match super::adapters::find_empirical_artifact_path(clause_id, contract, evidence, source_root) {
        Ok(path) => path,
        Err(error) => {
            diagnostics.push(error);
            return;
        }
    };
    let receipt: NormalFormReceipt = match super::super::read_json_bounded(&path) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("parse normal-form receipt {}: {error}", path.display()));
            return;
        }
    };
    if receipt.schema_version != "octet-extraction-normal-form/v1"
        || receipt.status != "passed"
        || receipt.final_program_blake3 != policy.final_program_blake3
        || receipt.rule_catalog_blake3 != policy.rule_catalog_blake3
        || receipt.side_condition_policy != policy.side_condition_policy
        || receipt.contexts.is_empty()
    {
        diagnostics.push("rule-relative normal form receipt failed policy compatibility checks".to_owned());
    }
}

pub(super) fn verify_effect_independence_adapter(
    profile: &super::EffectExtractionProfile,
    clause_id: &str,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let Some(policy) = &profile.optional_effect_independence else {
        diagnostics.push("optional effect-independence clause requested without policy".to_owned());
        return;
    };
    let Some(path) =
        find_proof_artifact_path(clause_id, "effect-independence.json", contract, evidence, source_root, diagnostics)
    else {
        return;
    };
    let receipt: EffectIndependenceReceipt = match super::super::read_json_bounded(&path) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("parse effect-independence receipt {}: {error}", path.display()));
            return;
        }
    };
    if receipt.schema_version != "octet-extraction-effect-independence/v1"
        || receipt.status != "passed"
        || !receipt.is_footprint_soundness_proved
        || !receipt.is_interchange_theorem_passed
        || receipt.extracted_core_blake3 != profile.subjects.extracted_core.blake3
        || receipt.driver_blake3 != profile.subjects.effect_driver.blake3
        || receipt.original_schedule_blake3 != policy.original_schedule_blake3
        || receipt.reordered_schedule_blake3 != policy.reordered_schedule_blake3
        || receipt.resource_mapping_blake3 != policy.resource_mapping_blake3
        || receipt.observer_identity != policy.observer_identity
        || receipt.relation_kind != policy.relation_kind
    {
        diagnostics.push("effect-independence certificate failed subject, footprint, or relation checks".to_owned());
    }
}

pub(super) fn find_proof_artifact_path(
    clause_id: &str,
    suffix: &str,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) -> Option<std::path::PathBuf> {
    let Some(clause) = contract.clauses.iter().find(|c| c.clause_id == clause_id) else {
        diagnostics.push(format!("missing formal clause `{clause_id}`"));
        return None;
    };
    for deck in &evidence.proof_decks {
        if !clause.allowed_evidence_ids.contains(&deck.evidence_id) {
            continue;
        }
        let deck_path = super::super::resolve_path(source_root, &deck.deck);
        let Ok(deck_json) = super::super::read_json_bounded::<serde_json::Value>(&deck_path) else {
            continue;
        };
        let Some(cards) = deck_json.get("cards").and_then(serde_json::Value::as_array) else {
            continue;
        };
        let artifact_root = super::super::resolve_path(source_root, &deck.artifact_root);
        for card in cards {
            let Some(card_id) = card.get("proof_or_property_id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if !clause.allowed_evidence_ids.iter().any(|allowed| allowed == card_id) {
                continue;
            }
            if let Some(identities) = card.get("artifact_blake3_identities").and_then(serde_json::Value::as_array) {
                for identity in identities {
                    if let Some(locator) = identity.get("artifact").and_then(serde_json::Value::as_str)
                        && locator.ends_with(suffix)
                    {
                        return Some(super::super::resolve_path(&artifact_root, locator));
                    }
                }
            }
        }
    }
    diagnostics.push(format!("clause `{clause_id}` proof card is missing `{suffix}` artifact"));
    None
}
